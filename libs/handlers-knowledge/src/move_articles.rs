use auth::Identity;
use axum::{Extension, Json, extract::State};
use handlers::{ApiError, AppState, DataEnvelope, Response, require_org_membership, resolve_actor};
use mestier_core::{KnowledgeArticleId, KnowledgeFolderId, MoveKnowledgeArticlesCommand};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::{MoveArticlesPath, unprocessable};

#[derive(Debug, Deserialize, ToSchema)]
pub struct MoveKnowledgeArticlesRequest {
    /// 1 to 100 articles of the organization.
    pub article_ids: Vec<KnowledgeArticleId>,
    /// Null moves the articles to the root.
    pub folder_id: Option<KnowledgeFolderId>,
}

#[derive(Debug, Serialize, PartialEq, ToSchema)]
pub struct MoveKnowledgeArticlesResponse {
    pub moved: u64,
}

#[utoipa::path(
    post,
    path = "/api/v1/organizations/{organization_id}/knowledge/articles/move",
    operation_id = "moveKnowledgeArticles",
    tag = crate::TAG,
    params(("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier")),
    request_body = MoveKnowledgeArticlesRequest,
    responses(
        (status = 200, description = "Every article is in the folder", body = inline(DataEnvelope<MoveKnowledgeArticlesResponse>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "The folder or one of the articles is not found, or held by another organization: nothing moves"),
        (status = 422, description = "More than 100 articles"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    MoveArticlesPath { organization_id }: MoveArticlesPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    Json(payload): Json<MoveKnowledgeArticlesRequest>,
) -> Result<Response<MoveKnowledgeArticlesResponse>, ApiError> {
    require_org_membership(&state, &identity, organization_id).await?;
    let (user_id, actor) = resolve_actor(&state, &identity).await?;
    let moved = state
        .usecase
        .acting_as(user_id)
        .move_knowledge_articles(MoveKnowledgeArticlesCommand {
            actor,
            organization_id,
            ids: payload.article_ids,
            folder_id: payload.folder_id,
        })
        .await?
        .map_err(unprocessable)?;
    Ok(Response::OK(MoveKnowledgeArticlesResponse { moved }))
}
