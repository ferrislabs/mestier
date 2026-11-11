use auth::Identity;
use axum::{Extension, Json, extract::State, response::IntoResponse};
use chrono::{DateTime, Utc};
use handlers::{ApiError, AppState, DataEnvelope, Response, require_org_membership, resolve_actor};
use http::StatusCode;
use mestier_core::{
    CreateKnowledgeDocumentCommand, DocumentLockCommand, DocumentRefusal,
    SaveDocumentContentCommand,
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::{
    ArticleContentPath, ArticleLockPath, DocumentsPath, response::KnowledgeArticleResponse,
    unprocessable,
};

#[derive(Debug)]
pub enum DocumentError {
    Api(ApiError),
    Refused(DocumentRefusal),
}

impl From<ApiError> for DocumentError {
    fn from(error: ApiError) -> Self {
        Self::Api(error)
    }
}

impl From<common::CoreError> for DocumentError {
    fn from(error: common::CoreError) -> Self {
        Self::Api(error.into())
    }
}

impl From<DocumentRefusal> for DocumentError {
    fn from(refusal: DocumentRefusal) -> Self {
        Self::Refused(refusal)
    }
}

fn conflict(code: &str, message: &str, extra: Value) -> axum::response::Response {
    let mut body = json!({ "code": code, "message": message, "status": 409 });
    if let (Some(body), Some(extra)) = (body.as_object_mut(), extra.as_object()) {
        body.extend(extra.clone());
    }
    (StatusCode::CONFLICT, Json(body)).into_response()
}

impl IntoResponse for DocumentError {
    fn into_response(self) -> axum::response::Response {
        match self {
            Self::Api(error) => error.into_response(),
            Self::Refused(DocumentRefusal::Locked(lock)) => conflict(
                "E_DOCUMENT_LOCKED",
                "This document is being edited by someone else",
                json!({ "holder_id": lock.holder.0, "expires_at": lock.expires_at }),
            ),
            Self::Refused(DocumentRefusal::LockLost) => conflict(
                "E_DOCUMENT_LOCK_LOST",
                "The edit lock was lost: take it again before saving",
                json!({ "reason": "lock_lost" }),
            ),
            Self::Refused(DocumentRefusal::RevisionConflict) => conflict(
                "E_DOCUMENT_REVISION_CONFLICT",
                "The document was changed since it was read: reload it",
                json!({ "reason": "revision_conflict" }),
            ),
            Self::Refused(DocumentRefusal::Invalid(error)) => {
                ApiError::UnprocessableEntity(error.to_string()).into_response()
            }
        }
    }
}

#[derive(Debug, Serialize, PartialEq, ToSchema)]
pub struct DocumentLockedResponse {
    pub code: String,
    pub message: String,
    pub status: u16,
    pub holder_id: Uuid,
    pub expires_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, PartialEq, ToSchema)]
pub struct DocumentConflictResponse {
    pub code: String,
    pub message: String,
    pub status: u16,
    /// `lock_lost` or `revision_conflict`.
    pub reason: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateKnowledgeDocumentRequest {
    /// 1 to 200 characters.
    pub title: String,
    /// Absent or null puts the document at the root.
    #[serde(default)]
    pub folder_id: Option<mestier_core::KnowledgeFolderId>,
    /// At most 20 tags of 1 to 40 characters, lowercased and deduplicated.
    #[serde(default)]
    pub tags: Vec<String>,
}

#[derive(Debug, Serialize, PartialEq, ToSchema)]
pub struct DocumentLockResponse {
    pub holder_id: Uuid,
    pub expires_at: DateTime<Utc>,
    pub revision: i32,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct SaveDocumentContentRequest {
    /// The blocks, in the Mestier block format: `{ "version": 1, "blocks": [...] }`.
    #[schema(value_type = Object)]
    pub content: Value,
    /// The revision the editor started from.
    pub revision: i32,
}

#[derive(Debug, Serialize, PartialEq, ToSchema)]
pub struct SavedDocumentResponse {
    pub revision: i32,
    pub updated_at: DateTime<Utc>,
}

#[utoipa::path(
    post,
    path = "/api/v1/organizations/{organization_id}/knowledge/articles/documents",
    operation_id = "createKnowledgeDocument",
    tag = crate::TAG,
    params(("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier")),
    request_body = CreateKnowledgeDocumentRequest,
    responses(
        (status = 201, description = "An empty document at revision 0", body = inline(DataEnvelope<KnowledgeArticleResponse>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Folder not found, or held by another organization"),
        (status = 422, description = "Empty or too long title or tag"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn create(
    DocumentsPath { organization_id }: DocumentsPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    Json(payload): Json<CreateKnowledgeDocumentRequest>,
) -> Result<Response<KnowledgeArticleResponse>, ApiError> {
    require_org_membership(&state, &identity, organization_id).await?;
    let (user_id, actor) = resolve_actor(&state, &identity).await?;

    let article = state
        .usecase
        .acting_as(user_id)
        .create_knowledge_document(CreateKnowledgeDocumentCommand {
            actor,
            organization_id,
            title: payload.title,
            tags: payload.tags,
            folder_id: payload.folder_id,
        })
        .await?
        .map_err(unprocessable)?;

    Ok(Response::Created(article.into()))
}

#[utoipa::path(
    post,
    path = "/api/v1/organizations/{organization_id}/knowledge/articles/{article_id}/lock",
    operation_id = "acquireKnowledgeDocumentLock",
    tag = crate::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
        ("article_id" = mestier_core::KnowledgeArticleId, Path, description = "Document identifier"),
    ),
    responses(
        (status = 200, description = "The caller holds the lock for 90 seconds. Calling it again renews it.", body = inline(DataEnvelope<DocumentLockResponse>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Document not found, or held by another organization"),
        (status = 409, description = "Someone else holds a live lock", body = DocumentLockedResponse),
    ),
    security(("bearer_auth" = []))
)]
pub async fn acquire_lock(
    ArticleLockPath {
        organization_id,
        article_id,
    }: ArticleLockPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<DocumentLockResponse>, DocumentError> {
    require_org_membership(&state, &identity, organization_id).await?;
    let (user_id, actor) = resolve_actor(&state, &identity).await?;

    let grant = state
        .usecase
        .acting_as(user_id)
        .acquire_document_lock(DocumentLockCommand {
            actor,
            organization_id,
            user_id,
            id: article_id,
        })
        .await??;

    Ok(Response::OK(DocumentLockResponse {
        holder_id: grant.lock.holder.0,
        expires_at: grant.lock.expires_at,
        revision: grant.revision,
    }))
}

#[utoipa::path(
    delete,
    path = "/api/v1/organizations/{organization_id}/knowledge/articles/{article_id}/lock",
    operation_id = "releaseKnowledgeDocumentLock",
    tag = crate::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
        ("article_id" = mestier_core::KnowledgeArticleId, Path, description = "Document identifier"),
    ),
    responses(
        (status = 204, description = "The lock is free. Nothing happens when the caller does not hold it."),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn release_lock(
    ArticleLockPath {
        organization_id,
        article_id,
    }: ArticleLockPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<StatusCode, ApiError> {
    require_org_membership(&state, &identity, organization_id).await?;
    let (user_id, actor) = resolve_actor(&state, &identity).await?;

    state
        .usecase
        .acting_as(user_id)
        .release_document_lock(DocumentLockCommand {
            actor,
            organization_id,
            user_id,
            id: article_id,
        })
        .await?;

    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    put,
    path = "/api/v1/organizations/{organization_id}/knowledge/articles/{article_id}/content",
    operation_id = "saveKnowledgeDocumentContent",
    tag = crate::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
        ("article_id" = mestier_core::KnowledgeArticleId, Path, description = "Document identifier"),
    ),
    request_body = SaveDocumentContentRequest,
    responses(
        (status = 200, description = "Saved: the revision went up by one and the Markdown body was rewritten", body = inline(DataEnvelope<SavedDocumentResponse>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Document, or an element added to it, not found or held by another organization"),
        (status = 409, description = "The lock is not the caller's (`lock_lost`) or the revision is stale (`revision_conflict`)", body = DocumentConflictResponse),
        (status = 422, description = "Invalid blocks, or beyond the limits"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn save_content(
    ArticleContentPath {
        organization_id,
        article_id,
    }: ArticleContentPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    Json(payload): Json<SaveDocumentContentRequest>,
) -> Result<Response<SavedDocumentResponse>, DocumentError> {
    require_org_membership(&state, &identity, organization_id).await?;
    let (user_id, actor) = resolve_actor(&state, &identity).await?;

    let saved = state
        .usecase
        .acting_as(user_id)
        .save_document_content(SaveDocumentContentCommand {
            actor,
            organization_id,
            user_id,
            id: article_id,
            content: payload.content,
            revision: payload.revision,
        })
        .await??;

    Ok(Response::OK(SavedDocumentResponse {
        revision: saved.revision,
        updated_at: saved.updated_at,
    }))
}
