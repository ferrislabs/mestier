use axum::{Router, middleware::from_fn_with_state};
use handlers::{AppState, auth::auth_middleware, rate_limit::rate_limit_middleware};

pub mod campaign;
mod request;
pub mod response;
pub mod schema;
pub mod survey;

pub const TAG: &str = "surveys";

pub use handlers::membership::require_org_membership;

pub fn router(state: &AppState) -> Router<AppState> {
    Router::new()
        .merge(survey::router(state))
        .merge(campaign::router(state))
        .merge(response::router(state))
        .layer(from_fn_with_state(state.clone(), rate_limit_middleware))
        .layer(from_fn_with_state(state.clone(), auth_middleware))
        .merge(response::public::router(state))
}
