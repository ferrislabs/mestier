use auth::Identity;
use axum::{Extension, extract::State};
use handlers::{ApiError, AppState, DataEnvelope, Response, require_org_membership, resolve_actor};
use mestier_core::GetKnowledgeCommand;

use crate::{ArticlePath, response::KnowledgeArticleResponse};

#[utoipa::path(
    get,
    path = "/api/v1/organizations/{organization_id}/knowledge/articles/{article_id}",
    operation_id = "getKnowledgeArticle",
    tag = crate::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
        ("article_id" = mestier_core::KnowledgeArticleId, Path, description = "Article identifier"),
    ),
    responses(
        (status = 200, description = "The article with the text read from its file and its links", body = inline(DataEnvelope<KnowledgeArticleResponse>)),
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
) -> Result<Response<KnowledgeArticleResponse>, ApiError> {
    require_org_membership(&state, &identity, organization_id).await?;
    let (_, actor) = resolve_actor(&state, &identity).await?;

    let found = state
        .usecase
        .get_knowledge_article(GetKnowledgeCommand {
            actor,
            organization_id,
            id: article_id,
        })
        .await?;

    let mut response = KnowledgeArticleResponse::from(found.article);
    response.links = Some(found.links.into_iter().map(Into::into).collect());
    Ok(Response::OK(response))
}
