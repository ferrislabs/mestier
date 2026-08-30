use axum::{Router, middleware::from_fn_with_state};
use axum_extra::routing::RouterExt;
use handlers::{AppState, auth::auth_middleware, rate_limit::rate_limit_middleware};

pub mod invitation;
pub mod member;
pub mod paths;
pub mod response;
pub mod role;

pub const TAG: &str = "members";

#[derive(Debug, serde::Serialize, PartialEq)]
pub struct EmptyResponse;

pub fn router(state: &AppState) -> Router<AppState> {
    Router::new()
        .typed_get(member::list::handler)
        .typed_post(member::create::handler)
        .typed_get(member::get_one::handler)
        .typed_get(member::permissions::handler)
        .typed_patch(member::update::handler)
        .typed_delete(member::soft_delete::handler)
        .typed_get(invitation::list::handler)
        .typed_post(invitation::create::handler)
        .typed_delete(invitation::revoke::handler)
        .typed_post(invitation::accept::handler)
        .typed_get(role::list::handler)
        .typed_post(role::create::handler)
        .typed_patch(role::update::handler)
        .typed_delete(role::delete::handler)
        .typed_get(role::list_for_member::handler)
        .typed_post(role::assign::handler)
        .typed_delete(role::unassign::handler)
        .layer(from_fn_with_state(state.clone(), rate_limit_middleware))
        .layer(from_fn_with_state(state.clone(), auth_middleware))
}
