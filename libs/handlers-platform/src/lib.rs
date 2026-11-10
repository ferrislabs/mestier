//! HTTP adapters for what sits above the organizations: product news, read by
//! every authenticated user and written from a back-office.

use auth::Identity;
use axum::{Router, middleware::from_fn_with_state};
use axum_extra::routing::{RouterExt, TypedPath};
use handlers::{ApiError, AppState, auth::auth_middleware, rate_limit::rate_limit_middleware};
use mestier_core::{NewsEditor, NewsError, NewsId};
use serde::Deserialize;

pub mod access;
pub mod admin_create;
pub mod admin_delete;
pub mod admin_list;
pub mod admin_update;
pub mod list;
pub mod request;
pub mod response;
pub mod seen;
pub mod unseen;

pub const TAG: &str = "news";

pub fn router(state: &AppState) -> Router<AppState> {
    Router::new()
        .typed_get(list::handler)
        .typed_get(unseen::handler)
        .typed_post(seen::handler)
        .typed_get(access::handler)
        .typed_get(admin_list::handler)
        .typed_post(admin_create::handler)
        .typed_put(admin_update::handler)
        .typed_delete(admin_delete::handler)
        .layer(from_fn_with_state(state.clone(), rate_limit_middleware))
        .layer(from_fn_with_state(state.clone(), auth_middleware))
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/news")]
pub struct NewsPath;

#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/news/unseen")]
pub struct UnseenPath;

#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/news/seen")]
pub struct SeenPath;

#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/news/access")]
pub struct AccessPath;

#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/admin/news")]
pub struct AdminNewsPath;

#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/admin/news/{news_id}")]
pub struct AdminNewsItemPath {
    pub news_id: NewsId,
}

/// The back-office needs the `mestier:admin` realm role on the caller's own
/// token; a delegated token (an assistant) or a client never has it.
pub(crate) fn news_editor(identity: &Identity) -> Result<NewsEditor, ApiError> {
    NewsEditor::from_realm_roles(identity.realm_roles()).ok_or(ApiError::Forbidden)
}

pub(crate) fn unprocessable(error: NewsError) -> ApiError {
    ApiError::UnprocessableEntity(error.to_string())
}
