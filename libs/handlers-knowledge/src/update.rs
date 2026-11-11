use auth::Identity;
use axum::{Extension, Json, extract::State};
use handlers::{ApiError, AppState, DataEnvelope, Response, require_org_membership, resolve_actor};
use mestier_core::UpdateKnowledgeCommand;
use serde::Deserialize;
use utoipa::ToSchema;

use crate::{ArticlePath, response::KnowledgeArticleResponse, unprocessable};

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateKnowledgeArticleRequest {
    /// Absent leaves it unchanged.
    #[serde(default)]
    pub title: Option<String>,
    /// The document as Markdown, converted to blocks. Refused on a file, and
    /// with 409 while someone else holds the edit lock. Absent leaves it unchanged.
    #[serde(default)]
    pub body: Option<String>,
    /// Replaces the whole list. Absent leaves it unchanged.
    #[serde(default)]
    pub tags: Option<Vec<String>>,
}

#[utoipa::path(
    patch,
    path = "/api/v1/organizations/{organization_id}/knowledge/articles/{article_id}",
    operation_id = "updateKnowledgeArticle",
    tag = crate::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
        ("article_id" = mestier_core::KnowledgeArticleId, Path, description = "Article identifier"),
    ),
    request_body = UpdateKnowledgeArticleRequest,
    responses(
        (status = 200, description = "The article, updated", body = inline(DataEnvelope<KnowledgeArticleResponse>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Article, or an element mentioned in the Markdown, not found or held by another organization"),
        (status = 409, description = "Someone else is editing the document"),
        (status = 422, description = "Empty or too long title, text or tag, invalid blocks, or a text sent for a file"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    ArticlePath {
        organization_id,
        article_id,
    }: ArticlePath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    Json(payload): Json<UpdateKnowledgeArticleRequest>,
) -> Result<Response<KnowledgeArticleResponse>, ApiError> {
    require_org_membership(&state, &identity, organization_id).await?;
    let (user_id, actor) = resolve_actor(&state, &identity).await?;

    let article = state
        .usecase
        .acting_as(user_id)
        .update_knowledge_article(UpdateKnowledgeCommand {
            actor,
            organization_id,
            user_id,
            id: article_id,
            title: payload.title,
            body: payload.body,
            tags: payload.tags,
        })
        .await?
        .map_err(unprocessable)?;

    Ok(Response::OK(article.into()))
}
