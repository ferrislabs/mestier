use auth::Identity;
use axum::{Extension, Json, extract::State};
use handlers::{ApiError, AppState, DataEnvelope, Response, require_org_membership, resolve_actor};
use http::StatusCode;
use mestier_core::{
    BrowseKnowledgeFolderCommand, CreateKnowledgeFolderCommand, DeleteKnowledgeFolderCommand,
    KnowledgeFolderId, ListKnowledgeFoldersCommand, OrganizationId, UpdateKnowledgeFolderCommand,
};
use serde::Deserialize;
use utoipa::ToSchema;

use crate::{
    FolderPath, FolderTreePath, FoldersPath, deserialize_present,
    response::{KnowledgeFolderResponse, KnowledgeFolderViewResponse},
    unprocessable,
};

async fn open(
    state: &AppState,
    identity: &Identity,
    organization_id: OrganizationId,
    id: Option<KnowledgeFolderId>,
) -> Result<Response<KnowledgeFolderViewResponse>, ApiError> {
    require_org_membership(state, identity, organization_id).await?;
    let (_, actor) = resolve_actor(state, identity).await?;
    let view = state
        .usecase
        .browse_knowledge_folder(BrowseKnowledgeFolderCommand {
            actor,
            organization_id,
            id,
        })
        .await?;
    Ok(Response::OK(view.into()))
}

#[utoipa::path(
    get,
    path = "/api/v1/organizations/{organization_id}/knowledge/folder-tree",
    operation_id = "listKnowledgeFolders",
    tag = crate::TAG,
    params(("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier")),
    responses(
        (status = 200, description = "Every folder of the drive, flat and by name: each one names its parent", body = inline(DataEnvelope<Vec<KnowledgeFolderResponse>>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn tree(
    FolderTreePath { organization_id }: FolderTreePath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<Vec<KnowledgeFolderResponse>>, ApiError> {
    require_org_membership(&state, &identity, organization_id).await?;
    let (_, actor) = resolve_actor(&state, &identity).await?;
    let folders = state
        .usecase
        .list_knowledge_folders(ListKnowledgeFoldersCommand {
            actor,
            organization_id,
        })
        .await?;
    Ok(Response::OK(folders.into_iter().map(Into::into).collect()))
}

#[utoipa::path(
    get,
    path = "/api/v1/organizations/{organization_id}/knowledge/folders",
    operation_id = "browseKnowledgeRoot",
    tag = crate::TAG,
    params(("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier")),
    responses(
        (status = 200, description = "The folders at the root of the drive", body = inline(DataEnvelope<KnowledgeFolderViewResponse>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn browse_root(
    FoldersPath { organization_id }: FoldersPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<KnowledgeFolderViewResponse>, ApiError> {
    open(&state, &identity, organization_id, None).await
}

#[utoipa::path(
    get,
    path = "/api/v1/organizations/{organization_id}/knowledge/folders/{folder_id}",
    operation_id = "browseKnowledgeFolder",
    tag = crate::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
        ("folder_id" = mestier_core::KnowledgeFolderId, Path, description = "Folder identifier"),
    ),
    responses(
        (status = 200, description = "The folder, its path from the root and its subfolders", body = inline(DataEnvelope<KnowledgeFolderViewResponse>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Folder not found, or held by another organization"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn browse(
    FolderPath {
        organization_id,
        folder_id,
    }: FolderPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<KnowledgeFolderViewResponse>, ApiError> {
    open(&state, &identity, organization_id, Some(folder_id)).await
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateKnowledgeFolderRequest {
    /// 1 to 100 characters, unique among its siblings whatever the case.
    pub name: String,
    /// Absent or null creates the folder at the root.
    #[serde(default)]
    pub parent_id: Option<KnowledgeFolderId>,
}

#[utoipa::path(
    post,
    path = "/api/v1/organizations/{organization_id}/knowledge/folders",
    operation_id = "createKnowledgeFolder",
    tag = crate::TAG,
    params(("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier")),
    request_body = CreateKnowledgeFolderRequest,
    responses(
        (status = 201, description = "The folder", body = inline(DataEnvelope<KnowledgeFolderResponse>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Parent not found, or held by another organization"),
        (status = 409, description = "A sibling already has this name"),
        (status = 422, description = "Empty or too long name, or more than 10 levels"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn create(
    FoldersPath { organization_id }: FoldersPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    Json(payload): Json<CreateKnowledgeFolderRequest>,
) -> Result<Response<KnowledgeFolderResponse>, ApiError> {
    require_org_membership(&state, &identity, organization_id).await?;
    let (user_id, actor) = resolve_actor(&state, &identity).await?;
    let folder = state
        .usecase
        .acting_as(user_id)
        .create_knowledge_folder(CreateKnowledgeFolderCommand {
            actor,
            organization_id,
            parent_id: payload.parent_id,
            name: payload.name,
        })
        .await?
        .map_err(unprocessable)?;
    Ok(Response::Created(folder.into()))
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateKnowledgeFolderRequest {
    /// Absent keeps the name.
    #[serde(default)]
    pub name: Option<String>,
    /// Absent keeps the parent; null moves the folder to the root.
    #[serde(default, deserialize_with = "deserialize_present")]
    #[schema(value_type = Option<mestier_core::KnowledgeFolderId>, nullable)]
    pub parent_id: Option<Option<KnowledgeFolderId>>,
}

#[utoipa::path(
    patch,
    path = "/api/v1/organizations/{organization_id}/knowledge/folders/{folder_id}",
    operation_id = "updateKnowledgeFolder",
    tag = crate::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
        ("folder_id" = mestier_core::KnowledgeFolderId, Path, description = "Folder identifier"),
    ),
    request_body = UpdateKnowledgeFolderRequest,
    responses(
        (status = 200, description = "The folder, renamed or moved", body = inline(DataEnvelope<KnowledgeFolderResponse>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Folder or new parent not found, or held by another organization"),
        (status = 409, description = "A sibling already has this name"),
        (status = 422, description = "Empty or too long name, a move inside itself, or more than 10 levels"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn update(
    FolderPath {
        organization_id,
        folder_id,
    }: FolderPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    Json(payload): Json<UpdateKnowledgeFolderRequest>,
) -> Result<Response<KnowledgeFolderResponse>, ApiError> {
    require_org_membership(&state, &identity, organization_id).await?;
    let (user_id, actor) = resolve_actor(&state, &identity).await?;
    let folder = state
        .usecase
        .acting_as(user_id)
        .update_knowledge_folder(UpdateKnowledgeFolderCommand {
            actor,
            organization_id,
            id: folder_id,
            name: payload.name,
            parent_id: payload.parent_id,
        })
        .await?
        .map_err(unprocessable)?;
    Ok(Response::OK(folder.into()))
}

#[utoipa::path(
    delete,
    path = "/api/v1/organizations/{organization_id}/knowledge/folders/{folder_id}",
    operation_id = "deleteKnowledgeFolder",
    tag = crate::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
        ("folder_id" = mestier_core::KnowledgeFolderId, Path, description = "Folder identifier"),
    ),
    responses(
        (status = 204, description = "The folder is deleted"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Folder not found, or held by another organization"),
        (status = 409, description = "The folder still holds a subfolder or an article"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn delete(
    FolderPath {
        organization_id,
        folder_id,
    }: FolderPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<StatusCode, ApiError> {
    require_org_membership(&state, &identity, organization_id).await?;
    let (user_id, actor) = resolve_actor(&state, &identity).await?;
    state
        .usecase
        .acting_as(user_id)
        .delete_knowledge_folder(DeleteKnowledgeFolderCommand {
            actor,
            organization_id,
            id: folder_id,
        })
        .await?
        .map_err(unprocessable)?;
    Ok(StatusCode::NO_CONTENT)
}
