use auth::Identity;
use axum::{Router, middleware::from_fn_with_state};
use handlers::{ApiError, AppState, auth::auth_middleware, rate_limit::rate_limit_middleware};
use mestier_core::{OrganizationId, Permissions};

pub mod closure_reason;
pub mod establishment;
pub mod paths;
pub mod promotion;
pub mod response;
pub mod training;

pub const TAG: &str = "cursus";

pub use handlers::membership::require_org_membership;

pub async fn require_view_cursus(
    state: &AppState,
    identity: &Identity,
    organization_id: OrganizationId,
) -> Result<(), ApiError> {
    let user = state
        .usecase
        .find_user_by_sub(identity.id())
        .await?
        .ok_or(ApiError::Forbidden)?;

    if state
        .usecase
        .find_membership(organization_id, user.id)
        .await?
        .is_none()
    {
        return Err(ApiError::Forbidden);
    }

    handlers::require_mcp_access(state, identity, organization_id).await?;

    let permissions = state
        .usecase
        .member_permissions(user.id, organization_id)
        .await?;

    if !permissions.contains(Permissions::VIEW_CURSUS) {
        return Err(ApiError::Forbidden);
    }

    Ok(())
}

pub fn router(state: &AppState) -> Router<AppState> {
    Router::new()
        .merge(establishment::router(state))
        .merge(training::router(state))
        .merge(promotion::router(state))
        .merge(closure_reason::router(state))
        .layer(from_fn_with_state(state.clone(), rate_limit_middleware))
        .layer(from_fn_with_state(state.clone(), auth_middleware))
}
