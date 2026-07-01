use auth::Identity;
use axum::{Router, middleware::from_fn_with_state, routing::get};
use axum_extra::routing::{RouterExt, TypedPath};
use handlers::{ApiError, AppState, auth::auth_middleware, rate_limit::rate_limit_middleware};
use mestier_core::{OrganizationId, User};

use crate::paths::{CurrentUserOrganizationsPath, OrganizationPath, OrganizationsPath};

pub mod create;
pub mod get_one;
pub mod list;
pub mod list_mine;
pub mod organization_context;
pub mod paths;
pub mod response;
pub mod soft_delete;
pub mod update;

pub const TAG: &str = "organizations";

#[derive(Debug, serde::Serialize, PartialEq)]
pub struct EmptyResponse;

async fn resolve_identity_user(state: &AppState, identity: &Identity) -> Result<User, ApiError> {
    state
        .usecase
        .find_user_by_sub(identity.id())
        .await?
        .ok_or(ApiError::Unauthorized)
}

async fn require_org_membership(
    state: &AppState,
    identity: &Identity,
    organization_id: OrganizationId,
) -> Result<User, ApiError> {
    let user = resolve_identity_user(state, identity).await?;
    let membership = state
        .usecase
        .find_membership(organization_id, user.id)
        .await?;

    if membership.is_none() {
        return Err(ApiError::Forbidden);
    }

    Ok(user)
}

pub fn router(state: &AppState) -> Router<AppState> {
    Router::new()
        .route(
            OrganizationsPath::PATH,
            get(list::handler).post(create::handler),
        )
        .route(
            OrganizationPath::PATH,
            get(get_one::handler)
                .patch(update::handler)
                .delete(soft_delete::handler),
        )
        .route(CurrentUserOrganizationsPath::PATH, get(list_mine::handler))
        .typed_get(organization_context::list::handler)
        .typed_post(organization_context::create::handler)
        .typed_patch(organization_context::update::handler)
        .typed_delete(organization_context::soft_delete::handler)
        .layer(from_fn_with_state(state.clone(), rate_limit_middleware))
        .layer(from_fn_with_state(state.clone(), auth_middleware))
}
