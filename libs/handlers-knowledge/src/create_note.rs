use auth::Identity;
use axum::{Extension, Json, extract::State};
use handlers::{ApiError, AppState, DataEnvelope, Response, require_org_membership, resolve_actor};
use mestier_core::CreateKnowledgeNoteCommand;
use serde::Deserialize;
use utoipa::ToSchema;

use crate::{NotesPath, response::KnowledgeArticleResponse, unprocessable};

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateKnowledgeNoteRequest {
    /// 1 to 200 characters.
    pub title: String,
    /// The document as Markdown, up to 100 000 characters, converted to blocks.
    pub body: String,
    /// At most 20 tags of 1 to 40 characters, lowercased and deduplicated.
    #[serde(default)]
    pub tags: Vec<String>,
    /// Absent or null puts the document at the root.
    #[serde(default)]
    pub folder_id: Option<mestier_core::KnowledgeFolderId>,
}

#[utoipa::path(
    post,
    path = "/api/v1/organizations/{organization_id}/knowledge/notes",
    operation_id = "createKnowledgeNote",
    tag = crate::TAG,
    params(("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier")),
    request_body = CreateKnowledgeNoteRequest,
    responses(
        (status = 201, description = "The document", body = inline(DataEnvelope<KnowledgeArticleResponse>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Folder, or an element mentioned in the Markdown, not found or held by another organization"),
        (status = 422, description = "Empty or too long title, text or tag, or blocks beyond the limits"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    NotesPath { organization_id }: NotesPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    Json(payload): Json<CreateKnowledgeNoteRequest>,
) -> Result<Response<KnowledgeArticleResponse>, ApiError> {
    require_org_membership(&state, &identity, organization_id).await?;
    let (user_id, actor) = resolve_actor(&state, &identity).await?;

    let article = state
        .usecase
        .acting_as(user_id)
        .create_knowledge_note(CreateKnowledgeNoteCommand {
            actor,
            organization_id,
            title: payload.title,
            body: payload.body,
            tags: payload.tags,
            folder_id: payload.folder_id,
        })
        .await?
        .map_err(unprocessable)?;

    Ok(Response::Created(article.into()))
}
