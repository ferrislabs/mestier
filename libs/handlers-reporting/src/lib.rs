use auth::Identity;
use axum::{Router, middleware::from_fn_with_state};
use axum_extra::routing::RouterExt;
use handlers::{ApiError, AppState, auth::auth_middleware, rate_limit::rate_limit_middleware};
use mestier_core::OrganizationId;

pub mod paths;
pub mod reporting;
pub mod response;

pub const TAG: &str = "reporting";

/// Belonging to the organization is enough to read its reports.
///
/// Deliberately not the stricter check the field routes make: these figures are
/// the foreman's view, and requiring an employee profile would lock out an
/// office manager who has no hourly rate. What they must never do is read
/// another organization's numbers, which is what this enforces.
async fn require_org_membership(
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

    Ok(())
}

pub fn router(state: &AppState) -> Router<AppState> {
    Router::new()
        .typed_get(reporting::profitability::handler)
        .typed_get(reporting::worked_hours::handler)
        .layer(from_fn_with_state(state.clone(), rate_limit_middleware))
        .layer(from_fn_with_state(state.clone(), auth_middleware))
}
