use axum::{Router, middleware::from_fn_with_state};
use axum_extra::routing::RouterExt;
use handlers::{AppState, auth::auth_middleware, rate_limit::rate_limit_middleware};

pub mod member;
pub mod paths;
pub mod response;

pub const TAG: &str = "members";

#[derive(Debug, serde::Serialize, PartialEq)]
pub struct EmptyResponse;

pub fn router(state: &AppState) -> Router<AppState> {
    Router::new()
        .typed_get(member::list::handler)
        .typed_post(member::create::handler)
        .typed_get(member::get_one::handler)
        .typed_patch(member::update::handler)
        .typed_delete(member::soft_delete::handler)
        .layer(from_fn_with_state(state.clone(), rate_limit_middleware))
        .layer(from_fn_with_state(state.clone(), auth_middleware))
}
