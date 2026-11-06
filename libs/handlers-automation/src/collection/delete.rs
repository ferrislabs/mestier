use auth::Identity;
use axum::{Extension, extract::State};
use handlers::{AppState, Response};

use crate::{
    error::AutomationError, paths::CollectionPath, require_manage_automation,
    workflow::delete::EmptyResponse,
};

#[utoipa::path(
    delete,
    path = "/api/v1/organizations/{organization_id}/automation/collections/{collection_id}",
    operation_id = "deleteCollection",
    tag = super::super::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
        ("collection_id" = uuid::Uuid, Path, description = "Collection identifier"),
    ),
    responses(
        (status = 204, description = "Collection and its history deleted; a workflow using it is not refused, it fails its next save and its next run with the collection named"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Collection not found in this organization"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    CollectionPath {
        organization_id,
        collection_id,
    }: CollectionPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<EmptyResponse>, AutomationError> {
    require_manage_automation(&state, &identity, organization_id).await?;

    state
        .usecase
        .delete_collection(organization_id, collection_id)
        .await?;

    Ok(Response::NoContent)
}
