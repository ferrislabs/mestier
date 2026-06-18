use auth::Identity;
use axum::{Router, middleware::from_fn_with_state};
use axum_extra::routing::RouterExt;
use handlers::{ApiError, AppState, auth::auth_middleware, rate_limit::rate_limit_middleware};

pub mod paths;
pub mod response;
pub mod user;
pub mod webhook;

pub const TAG: &str = "users";

#[derive(Debug, serde::Serialize, PartialEq)]
pub struct EmptyResponse;

/// Verify the caller is a known Mestier user (has a local record).
/// Returns [`ApiError::Forbidden`] if the sub is not found in the local copy.
///
/// This is the minimum authz guard for user-management endpoints: the caller
/// must be authenticated and present in the system. A future task may add a
/// role/scope check here.
async fn require_known_user(state: &AppState, identity: &Identity) -> Result<(), ApiError> {
    state
        .usecase
        .find_user_by_sub(identity.id())
        .await?
        .ok_or(ApiError::Forbidden)?;
    Ok(())
}

pub fn router(state: &AppState) -> Router<AppState> {
    Router::new()
        .typed_get(user::list::handler)
        .typed_post(user::create::handler)
        .typed_patch(user::update::handler)
        .typed_delete(user::disable::handler)
        .layer(from_fn_with_state(state.clone(), rate_limit_middleware))
        .layer(from_fn_with_state(state.clone(), auth_middleware))
}

/// Unauthenticated router for the FerrisKey webhook receiver.
///
/// This router intentionally does NOT carry `auth_middleware`. The only
/// credential is the HMAC-SHA256 signature verified inside the handler.
pub fn webhook_router(_state: &AppState) -> Router<AppState> {
    Router::new().route(
        "/api/v1/webhooks/ferriskey",
        axum::routing::post(webhook::handler),
    )
}
