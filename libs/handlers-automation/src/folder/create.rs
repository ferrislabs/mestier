use auth::Identity;
use axum::{Extension, Json, extract::State};
use handlers::{ApiError, AppState, DataEnvelope, Response};
use mestier_core::{CreateFolderCommand, FolderName};
use serde::Deserialize;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::{paths::FoldersPath, require_manage_automation, response::FolderResponse};

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateFolderRequest {
    pub name: String,
    #[serde(default)]
    pub parent_id: Option<Uuid>,
}

#[utoipa::path(
    post,
    path = "/api/v1/organizations/{organization_id}/automation/folders",
    operation_id = "createWorkflowFolder",
    tag = super::super::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
    ),
    request_body = CreateFolderRequest,
    responses(
        (status = 201, description = "Folder created", body = inline(DataEnvelope<FolderResponse>)),
        (status = 400, description = "The name is empty or longer than 80 characters"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 409, description = "The parent folder does not exist, or a sibling already has this name (case-insensitive)"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    path: FoldersPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    Json(payload): Json<CreateFolderRequest>,
) -> Result<Response<FolderResponse>, ApiError> {
    require_manage_automation(&state, &identity, path.organization_id).await?;

    let name = FolderName::try_from(payload.name)
        .map_err(|error| ApiError::Validation(error.to_string()))?;
    let folder = state
        .usecase
        .create_workflow_folder(CreateFolderCommand {
            org_id: path.organization_id,
            name,
            parent_id: payload.parent_id,
        })
        .await?;

    Ok(Response::Created(FolderResponse::from(folder)))
}
