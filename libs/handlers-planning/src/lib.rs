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

pub mod paths;
pub mod planning;
pub mod response;
pub mod work_order;
pub mod work_time;

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

/// Rejects an employee that belongs to another organization.
///
/// Duplicated from `handlers-reference`, which carries the same guard for
/// absences: the two crates are separate HTTP adapters and the repository
/// already duplicates `require_org_membership` the same way. Ten lines of
/// honest duplication beat a shared module built for two occurrences.
pub(crate) async fn require_employee_target(
    state: &AppState,
    organization_id: OrganizationId,
    employee_id: mestier_core::EmployeeId,
) -> Result<(), ApiError> {
    let employee = state.usecase.get_employee(employee_id).await?;
    if employee.organization_id != organization_id {
        return Err(ApiError::Forbidden);
    }

    Ok(())
}

pub fn router(state: &AppState) -> Router<AppState> {
    Router::new()
        .merge(work_order::router(state))
        .merge(work_time::router(state))
        .merge(planning::router(state))
        .layer(from_fn_with_state(state.clone(), rate_limit_middleware))
        .layer(from_fn_with_state(state.clone(), auth_middleware))
}
