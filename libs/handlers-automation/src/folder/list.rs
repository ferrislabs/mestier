use auth::Identity;
use axum::{Extension, extract::State};
use handlers::{ApiError, AppState, DataEnvelope, Response};

use crate::{paths::FoldersPath, require_view_automation, response::FolderResponse};

#[utoipa::path(
    get,
    path = "/api/v1/organizations/{organization_id}/automation/folders",
    operation_id = "listWorkflowFolders",
    tag = super::super::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
    ),
    responses(
        (status = 200, description = "Every folder of this organization as a flat list, sorted by name", body = inline(DataEnvelope<Vec<FolderResponse>>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    path: FoldersPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<Vec<FolderResponse>>, ApiError> {
    require_view_automation(&state, &identity, path.organization_id).await?;

    let folders = state
        .usecase
        .list_workflow_folders(path.organization_id)
        .await?;
    let body: Vec<FolderResponse> = folders.into_iter().map(FolderResponse::from).collect();

    Ok(Response::OK(body))
}
