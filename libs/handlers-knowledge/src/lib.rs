//! HTTP adapters for the knowledge base: files and notes an organization
//! keeps, searched by text and tags, and linked to projects, customers and
//! quotes.

use axum::{
    Router,
    extract::DefaultBodyLimit,
    middleware::from_fn_with_state,
    routing::{get, post},
};
use axum_extra::routing::{RouterExt, TypedPath};
use handlers::{ApiError, AppState, auth::auth_middleware, rate_limit::rate_limit_middleware};
use mestier_core::{KnowledgeArticleId, KnowledgeError, KnowledgeResourceType, OrganizationId};
use serde::Deserialize;
use uuid::Uuid;

pub mod create_note;
pub mod delete;
pub mod download;
pub mod get_one;
pub mod link;
pub mod list;
pub mod response;
pub mod update;
pub mod upload;

pub const TAG: &str = "knowledge";

pub fn router(state: &AppState) -> Router<AppState> {
    let upload_limit = mestier_core::MAX_FILE_BYTES.saturating_add(1);

    Router::new()
        .route(
            ArticlesPath::PATH,
            get(list::handler)
                .merge(post(upload::handler).layer(DefaultBodyLimit::max(upload_limit))),
        )
        .typed_post(create_note::handler)
        .typed_get(get_one::handler)
        .typed_patch(update::handler)
        .typed_delete(delete::handler)
        .typed_get(download::handler)
        .typed_post(link::add)
        .typed_delete(link::remove)
        .layer(from_fn_with_state(state.clone(), rate_limit_middleware))
        .layer(from_fn_with_state(state.clone(), auth_middleware))
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/organizations/{organization_id}/knowledge/articles")]
pub struct ArticlesPath {
    pub organization_id: OrganizationId,
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/organizations/{organization_id}/knowledge/notes")]
pub struct NotesPath {
    pub organization_id: OrganizationId,
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/organizations/{organization_id}/knowledge/articles/{article_id}")]
pub struct ArticlePath {
    pub organization_id: OrganizationId,
    pub article_id: KnowledgeArticleId,
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/organizations/{organization_id}/knowledge/articles/{article_id}/download")]
pub struct DownloadPath {
    pub organization_id: OrganizationId,
    pub article_id: KnowledgeArticleId,
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/organizations/{organization_id}/knowledge/articles/{article_id}/links")]
pub struct LinksPath {
    pub organization_id: OrganizationId,
    pub article_id: KnowledgeArticleId,
}

#[derive(TypedPath, Deserialize)]
#[typed_path(
    "/api/v1/organizations/{organization_id}/knowledge/articles/{article_id}/links/{resource_type}/{resource_id}"
)]
pub struct LinkPath {
    pub organization_id: OrganizationId,
    pub article_id: KnowledgeArticleId,
    pub resource_type: KnowledgeResourceType,
    pub resource_id: Uuid,
}

pub(crate) fn unprocessable(error: KnowledgeError) -> ApiError {
    ApiError::UnprocessableEntity(error.to_string())
}
