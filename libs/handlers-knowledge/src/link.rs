use auth::Identity;
use axum::{Extension, Json, extract::State, http::StatusCode};
use handlers::{ApiError, AppState, DataEnvelope, Response, require_org_membership, resolve_actor};
use mestier_core::{KnowledgeResource, KnowledgeResourceType, LinkKnowledgeCommand};
use serde::Deserialize;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::{LinkPath, LinksPath, response::KnowledgeLinkResponse};

#[derive(Debug, Deserialize, ToSchema)]
pub struct LinkKnowledgeRequest {
    pub resource_type: KnowledgeResourceType,
    pub resource_id: Uuid,
}

#[utoipa::path(
    post,
    path = "/api/v1/organizations/{organization_id}/knowledge/articles/{article_id}/links",
    operation_id = "linkKnowledgeArticle",
    tag = crate::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
        ("article_id" = mestier_core::KnowledgeArticleId, Path, description = "Article identifier"),
    ),
    request_body = LinkKnowledgeRequest,
    responses(
        (status = 200, description = "The link. Linking twice keeps one link", body = inline(DataEnvelope<KnowledgeLinkResponse>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Article or resource not found, or held by another organization"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn add(
    LinksPath {
        organization_id,
        article_id,
    }: LinksPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    Json(payload): Json<LinkKnowledgeRequest>,
) -> Result<Response<KnowledgeLinkResponse>, ApiError> {
    require_org_membership(&state, &identity, organization_id).await?;
    let (user_id, actor) = resolve_actor(&state, &identity).await?;

    let link = state
        .usecase
        .acting_as(user_id)
        .link_knowledge_article(LinkKnowledgeCommand {
            actor,
            organization_id,
            id: article_id,
            resource: KnowledgeResource {
                resource_type: payload.resource_type,
                id: payload.resource_id,
            },
        })
        .await?;

    Ok(Response::OK(link.into()))
}

#[utoipa::path(
    delete,
    path = "/api/v1/organizations/{organization_id}/knowledge/articles/{article_id}/links/{resource_type}/{resource_id}",
    operation_id = "unlinkKnowledgeArticle",
    tag = crate::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
        ("article_id" = mestier_core::KnowledgeArticleId, Path, description = "Article identifier"),
        ("resource_type" = KnowledgeResourceType, Path, description = "project, customer or quote"),
        ("resource_id" = uuid::Uuid, Path, description = "Resource identifier"),
    ),
    responses(
        (status = 204, description = "The link is gone. Unlinking a link that does not exist is not an error"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Article not found, or held by another organization"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn remove(
    LinkPath {
        organization_id,
        article_id,
        resource_type,
        resource_id,
    }: LinkPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<StatusCode, ApiError> {
    require_org_membership(&state, &identity, organization_id).await?;
    let (user_id, actor) = resolve_actor(&state, &identity).await?;

    state
        .usecase
        .acting_as(user_id)
        .unlink_knowledge_article(LinkKnowledgeCommand {
            actor,
            organization_id,
            id: article_id,
            resource: KnowledgeResource {
                resource_type,
                id: resource_id,
            },
        })
        .await?;

    Ok(StatusCode::NO_CONTENT)
}
