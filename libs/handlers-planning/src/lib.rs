//! HTTP adapters for the planning module.
//!
//! The crate is split by aggregate, one submodule each: work orders, absences,
//! working time, and the planning read model. Every submodule exposes its own
//! `router(state)`, which [`router`] merges.
//!
//! Adding an aggregate therefore touches two lines of this file — a `pub mod`
//! and a `.merge(...)`. Those two lines are the only place where otherwise
//! independent workstreams collide, so this file is owned by whoever
//! integrates them: a workstream reports the lines it needs rather than
//! editing them itself.

use auth::Identity;
use axum::{Router, middleware::from_fn_with_state};
use handlers::{ApiError, AppState, auth::auth_middleware, rate_limit::rate_limit_middleware};
use mestier_core::OrganizationId;

pub mod absence;
pub mod paths;
pub mod response;
pub mod work_order;

pub const TAG: &str = "planning";

/// Rejects an identity that does not belong to the organization.
///
/// The planning module deliberately stops at membership for now: every member
/// sees the whole organization's planning. Finer bits will slot in here later
/// without a migration, since `roles.permissions` is already a bitfield.
pub async fn require_org_membership(
    state: &AppState,
    identity: &Identity,
    organization_id: OrganizationId,
) -> Result<(), ApiError> {
    let user = state
        .usecase
        .find_user_by_sub(identity.id())
        .await?
        .ok_or(ApiError::Forbidden)?;
    let membership = state
        .usecase
        .find_membership(organization_id, user.id)
        .await?;

    if membership.is_none() {
        return Err(ApiError::Forbidden);
    }

    Ok(())
}

pub fn router(state: &AppState) -> Router<AppState> {
    Router::new()
        .merge(work_order::router(state))
        .merge(absence::router(state))
        .layer(from_fn_with_state(state.clone(), rate_limit_middleware))
        .layer(from_fn_with_state(state.clone(), auth_middleware))
}
