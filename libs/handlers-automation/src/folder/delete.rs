use auth::Identity;
use axum::{Extension, extract::State};
use handlers::{AppState, Response};

use crate::{
    error::AutomationError, paths::FolderPath, require_manage_automation,
    workflow::delete::EmptyResponse,
};

#[utoipa::path(
    delete,
    path = "/api/v1/organizations/{organization_id}/automation/folders/{folder_id}",
    operation_id = "deleteWorkflowFolder",
    tag = super::super::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
        ("folder_id" = uuid::Uuid, Path, description = "Folder identifier"),
    ),
    responses(
        (status = 204, description = "Folder deleted"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Folder not found"),
        (status = 409, description = "E_FOLDER_NOT_EMPTY: the folder still holds a sub-folder or a workflow (archived ones included)"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    FolderPath {
        organization_id,
        folder_id,
    }: FolderPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<EmptyResponse>, AutomationError> {
    require_manage_automation(&state, &identity, organization_id).await?;

    state
        .usecase
        .delete_workflow_folder(organization_id, folder_id)
        .await?;

    Ok(Response::NoContent)
}
