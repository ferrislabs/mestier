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

use axum::{Router, middleware::from_fn_with_state};
use handlers::{ApiError, AppState, auth::auth_middleware, rate_limit::rate_limit_middleware};
use mestier_core::OrganizationId;

pub mod assignment_report;
pub mod doe;
pub mod doe_model;
pub mod paths;
pub mod planning;
pub mod project;
pub mod project_template;
pub mod recap_pdf;
pub mod response;
pub mod task;
pub mod task_comment;
pub mod task_label;
pub mod task_recurrence;
pub mod work_time;

pub const TAG: &str = "planning";

pub use handlers::membership::require_org_membership;

/// Rejects an employee that belongs to another organization.
///
/// Duplicated from `handlers-reference`, which carries the same guard for
/// absences: the two crates are separate HTTP adapters and the repository
/// already duplicates `require_org_membership` the same way. Ten lines of
/// honest duplication beat a shared module built for two occurrences.
/// Loads the seat and checks the caller belongs to *its* organization.
///
/// The work-time routes are addressed by bare `member_id` now, so there is no
/// organization in the path to check against — reading one from there is what
/// turns a bare id into a cross-tenant IDOR. Returns the organization so the
/// caller does not have to load the seat twice.
pub(crate) async fn require_member_target(
    state: &AppState,
    identity: &auth::Identity,
    member_id: mestier_core::MemberId,
) -> Result<OrganizationId, ApiError> {
    let member = state
        .usecase
        .find_member_seat(member_id)
        .await?
        .ok_or(ApiError::NotFound)?;

    require_org_membership(state, identity, member.organization_id).await?;

    Ok(member.organization_id)
}

pub fn router(state: &AppState) -> Router<AppState> {
    Router::new()
        .merge(assignment_report::router(state))
        .merge(task::router(state))
        .merge(task_comment::router(state))
        .merge(task_label::router(state))
        .merge(task_recurrence::router(state))
        .merge(work_time::router(state))
        .merge(planning::router(state))
        .merge(project::router(state))
        .merge(project_template::router(state))
        .merge(doe::router(state))
        .merge(doe_model::router(state))
        .merge(recap_pdf::router(state))
        .layer(from_fn_with_state(state.clone(), rate_limit_middleware))
        .layer(from_fn_with_state(state.clone(), auth_middleware))
}
