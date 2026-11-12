//! HTTP adapters for what sits above the organizations: product news, read by
//! every authenticated user and written from a back-office.

use auth::Identity;
use axum::{Router, middleware::from_fn_with_state};
use axum_extra::routing::{RouterExt, TypedPath};
use handlers::{ApiError, AppState, auth::auth_middleware, rate_limit::rate_limit_middleware};
use mestier_core::{FeatureFlag, NewsError, NewsId, OrganizationId, PlatformAdmin};
use serde::Deserialize;

pub mod access;
pub mod admin_create;
pub mod admin_delete;
pub mod admin_feature_catalogue;
pub mod admin_list;
pub mod admin_organization;
pub mod admin_organizations;
pub mod admin_set_feature;
pub mod admin_stats;
pub mod admin_update;
pub mod admin_users;
pub mod list;
pub mod request;
pub mod response;
pub mod seen;
pub mod unseen;

pub const TAG: &str = "news";
pub const ADMIN_TAG: &str = "admin";

pub fn router(state: &AppState) -> Router<AppState> {
    Router::new()
        .typed_get(list::handler)
        .typed_get(unseen::handler)
        .typed_post(seen::handler)
        .typed_get(access::handler)
        .typed_get(admin_stats::handler)
        .typed_get(admin_feature_catalogue::handler)
        .typed_get(admin_organizations::handler)
        .typed_get(admin_organization::handler)
        .typed_put(admin_set_feature::handler)
        .typed_get(admin_users::handler)
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
#[typed_path("/api/v1/admin/access")]
pub struct AccessPath;

#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/admin/stats")]
pub struct AdminStatsPath;

#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/admin/features")]
pub struct AdminFeaturesPath;

#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/admin/organizations")]
pub struct AdminOrganizationsPath;

#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/admin/organizations/{organization_id}")]
pub struct AdminOrganizationPath {
    pub organization_id: OrganizationId,
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/admin/organizations/{organization_id}/features/{flag}")]
pub struct AdminOrganizationFeaturePath {
    pub organization_id: OrganizationId,
    pub flag: String,
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/admin/users")]
pub struct AdminUsersPath;

#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/admin/news")]
pub struct AdminNewsPath;

#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/admin/news/{news_id}")]
pub struct AdminNewsItemPath {
    pub news_id: NewsId,
}

pub(crate) fn platform_admin(identity: &Identity) -> Result<PlatformAdmin, ApiError> {
    PlatformAdmin::from_realm_roles(identity.realm_roles()).ok_or(ApiError::Forbidden)
}

pub(crate) fn feature_flag(key: &str) -> Result<FeatureFlag, ApiError> {
    FeatureFlag::try_from(key).map_err(|error| ApiError::UnprocessableEntity(error.to_string()))
}

pub(crate) fn unprocessable(error: NewsError) -> ApiError {
    ApiError::UnprocessableEntity(error.to_string())
}

#[derive(Debug, Deserialize, utoipa::IntoParams)]
#[into_params(parameter_in = Query)]
pub struct SearchParams {
    #[serde(default)]
    pub search: Option<String>,
}
