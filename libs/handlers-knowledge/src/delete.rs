use auth::Identity;
use axum::{Extension, extract::State, http::StatusCode};
use handlers::{ApiError, AppState, require_org_membership, resolve_actor};
use mestier_core::DeleteKnowledgeCommand;

use crate::ArticlePath;

#[utoipa::path(
    delete,
    path = "/api/v1/organizations/{organization_id}/knowledge/articles/{article_id}",
    operation_id = "deleteKnowledgeArticle",
    tag = crate::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
        ("article_id" = mestier_core::KnowledgeArticleId, Path, description = "Article identifier"),
    ),
    responses(
        (status = 204, description = "Article, its links and its stored file deleted; no project, customer or quote is touched"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Article not found, or held by another organization"),
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
) -> Result<StatusCode, ApiError> {
    require_org_membership(&state, &identity, organization_id).await?;
    let (user_id, actor) = resolve_actor(&state, &identity).await?;

    let removed = state
        .usecase
        .acting_as(user_id)
        .delete_knowledge_article(DeleteKnowledgeCommand {
            actor,
            organization_id,
            id: article_id,
        })
        .await?;

    if let Some(file) = removed.file
        && let Err(error) = state.file_storage.delete(&file.key).await
    {
        tracing::warn!(key = %file.key, error = %error, "could not remove a deleted knowledge file");
    }

    Ok(StatusCode::NO_CONTENT)
}
