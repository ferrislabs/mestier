use auth::Identity;
use axum::{Router, middleware::from_fn_with_state};
use axum_extra::routing::RouterExt;
use handlers::{ApiError, AppState, auth::auth_middleware, rate_limit::rate_limit_middleware};
use mestier_core::{OrganizationId, Permissions};

pub mod paths;
pub mod reporting;
pub mod response;

pub const TAG: &str = "reporting";

/// Membership is the outer gate, `VIEW_REPORTS` is the inner one (#306):
/// belonging to the organization used to be enough to read its reports,
/// but these figures carry planned minutes *and* what people cost, and
/// reading either is not everybody's by virtue of being in the
/// organization — that gap is the reason #283 exists at all.
///
/// A caller from another organization is still rejected before their
/// bits are ever consulted, same rule #305 holds everywhere else.
///
/// Returns the caller's aggregated [`Permissions`] so a handler can
/// additionally check `VIEW_COST` to decide whether to redact the money
/// fields, without a second round trip — see
/// `reporting::profitability::handler`.
async fn require_view_reports(
    state: &AppState,
    identity: &Identity,
    organization_id: OrganizationId,
) -> Result<Permissions, ApiError> {
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

    let permissions = state
        .usecase
        .member_permissions(user.id, organization_id)
        .await?;

    if !permissions.contains(Permissions::VIEW_REPORTS) {
        return Err(ApiError::Forbidden);
    }

    Ok(permissions)
}

pub fn router(state: &AppState) -> Router<AppState> {
    Router::new()
        .typed_get(reporting::profitability::handler)
        .typed_get(reporting::worked_hours::handler)
        .layer(from_fn_with_state(state.clone(), rate_limit_middleware))
        .layer(from_fn_with_state(state.clone(), auth_middleware))
}
