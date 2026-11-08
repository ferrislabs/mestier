use auth::Identity;
use axum::{
    Extension, Json, Router,
    extract::State,
    response::{IntoResponse, Response as AxumResponse},
};
use axum_extra::routing::{RouterExt, TypedPath};
use chrono::{DateTime, Utc};
use handlers::{ApiError, AppState, DataEnvelope, Response, resolve_actor};
use http::StatusCode;
use mestier_core::{
    AddMemberDocumentCommand, DeleteMemberDocumentCommand, DocumentKind,
    ListMemberDocumentsCommand, MemberDocument, MemberDocumentError, MemberDocumentId, MemberId,
    OrganizationId,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use utoipa::ToSchema;

use crate::EmptyResponse;

pub fn router(_state: &AppState) -> Router<AppState> {
    Router::new()
        .typed_get(list_documents)
        .typed_post(add_document)
        .typed_delete(delete_document)
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/organizations/{organization_id}/member-documents")]
pub struct OrganizationDocumentsPath {
    pub organization_id: OrganizationId,
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/organizations/{organization_id}/members/{member_id}/documents")]
pub struct MemberDocumentsPath {
    pub organization_id: OrganizationId,
    pub member_id: MemberId,
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/organizations/{organization_id}/members/{member_id}/documents/{document_id}")]
pub struct MemberDocumentPath {
    pub organization_id: OrganizationId,
    pub member_id: MemberId,
    pub document_id: MemberDocumentId,
}

/// The file is uploaded first through `POST /files`; its `key` goes in `file_key`.
#[derive(Debug, Deserialize, ToSchema)]
pub struct AddMemberDocumentRequest {
    pub name: String,
    pub kind: DocumentKind,
    pub file_key: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, ToSchema)]
pub struct MemberDocumentResponse {
    pub id: MemberDocumentId,
    pub organization_id: OrganizationId,
    pub member_id: MemberId,
    pub name: String,
    pub kind: DocumentKind,
    pub file_key: String,
    pub added_at: DateTime<Utc>,
}

impl From<MemberDocument> for MemberDocumentResponse {
    fn from(value: MemberDocument) -> Self {
        Self {
            id: value.id,
            organization_id: value.organization_id,
            member_id: value.member_id,
            name: value.name.as_str().to_owned(),
            kind: value.kind,
            file_key: value.file_key.as_str().to_owned(),
            added_at: value.added_at,
        }
    }
}

#[derive(Debug)]
pub enum DocumentFailure {
    Api(ApiError),
    Refused(MemberDocumentError),
}

impl From<ApiError> for DocumentFailure {
    fn from(error: ApiError) -> Self {
        Self::Api(error)
    }
}

impl From<MemberDocumentError> for DocumentFailure {
    fn from(error: MemberDocumentError) -> Self {
        Self::Refused(error)
    }
}

impl IntoResponse for DocumentFailure {
    fn into_response(self) -> AxumResponse {
        match self {
            Self::Api(error) => error.into_response(),
            Self::Refused(error) => {
                let status = StatusCode::UNPROCESSABLE_ENTITY;
                (
                    status,
                    Json(json!({
                        "code": "E_UNPROCESSABLE_ENTITY",
                        "message": error.to_string(),
                        "status": status.as_u16(),
                        "details": { "field": error.field() },
                    })),
                )
                    .into_response()
            }
        }
    }
}

#[utoipa::path(
    get,
    path = "/api/v1/organizations/{organization_id}/member-documents",
    operation_id = "listMemberDocuments",
    tag = super::TAG,
    params(("organization_id" = OrganizationId, Path, description = "Organization identifier")),
    responses(
        (status = 200, description = "Every member document of the organization", body = inline(DataEnvelope<Vec<MemberDocumentResponse>>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn list_documents(
    path: OrganizationDocumentsPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<Vec<MemberDocumentResponse>>, ApiError> {
    let (user_id, actor) = resolve_actor(&state, &identity).await?;
    let documents = state
        .usecase
        .acting_as(user_id)
        .list_member_documents(ListMemberDocumentsCommand {
            actor,
            organization_id: path.organization_id,
        })
        .await?;

    Ok(Response::OK(
        documents.into_iter().map(Into::into).collect(),
    ))
}

#[utoipa::path(
    post,
    path = "/api/v1/organizations/{organization_id}/members/{member_id}/documents",
    operation_id = "addMemberDocument",
    tag = super::TAG,
    params(
        ("organization_id" = OrganizationId, Path, description = "Organization identifier"),
        ("member_id" = MemberId, Path, description = "Member identifier"),
    ),
    request_body = AddMemberDocumentRequest,
    responses(
        (status = 201, description = "Document recorded", body = inline(DataEnvelope<MemberDocumentResponse>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Member not found in the organization"),
        (status = 422, description = "The document is refused, the field is named in details"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn add_document(
    path: MemberDocumentsPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    Json(payload): Json<AddMemberDocumentRequest>,
) -> Result<Response<MemberDocumentResponse>, DocumentFailure> {
    let (user_id, actor) = resolve_actor(&state, &identity).await?;
    let document = state
        .usecase
        .acting_as(user_id)
        .add_member_document(AddMemberDocumentCommand {
            actor,
            organization_id: path.organization_id,
            member_id: path.member_id,
            name: payload.name,
            kind: payload.kind,
            file_key: payload.file_key,
        })
        .await
        .map_err(ApiError::from)??;

    Ok(Response::Created(document.into()))
}

#[utoipa::path(
    delete,
    path = "/api/v1/organizations/{organization_id}/members/{member_id}/documents/{document_id}",
    operation_id = "deleteMemberDocument",
    tag = super::TAG,
    params(
        ("organization_id" = OrganizationId, Path, description = "Organization identifier"),
        ("member_id" = MemberId, Path, description = "Member identifier"),
        ("document_id" = MemberDocumentId, Path, description = "Document identifier"),
    ),
    responses(
        (status = 204, description = "Document row deleted, the stored file is left untouched"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Document not found for this member in the organization"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn delete_document(
    path: MemberDocumentPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<EmptyResponse>, ApiError> {
    let (user_id, actor) = resolve_actor(&state, &identity).await?;
    state
        .usecase
        .acting_as(user_id)
        .delete_member_document(DeleteMemberDocumentCommand {
            actor,
            organization_id: path.organization_id,
            member_id: path.member_id,
            document_id: path.document_id,
        })
        .await?;

    Ok(Response::NoContent)
}
