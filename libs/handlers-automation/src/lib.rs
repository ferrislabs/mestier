//! HTTP adapters for the automation module (#203): the connector and event
//! catalogues, credentials, workflows, runs and settings — everything the
//! workflow engine (chantier A/B) exposes, reachable until now only from
//! Rust.
//!
//! Split by aggregate, one submodule each, mirroring `handlers-planning`:
//! every submodule exposes its own `router(state)`, which [`router`] merges
//! before the shared rate-limit/auth middleware is applied once.

use auth::Identity;
use axum::{Router, middleware::from_fn_with_state};
use handlers::{ApiError, AppState, auth::auth_middleware, rate_limit::rate_limit_middleware};
use mestier_core::{OrganizationId, Permissions};

pub mod catalogue;
pub mod credential;
pub mod paths;
pub mod response;
pub mod run;
pub mod settings;
pub mod workflow;

pub const TAG: &str = "automation";

/// Membership is the outer gate, `VIEW_AUTOMATION` the inner one (#493):
/// belonging to the organization used to be enough to read every credential,
/// workflow and run in it — no permission bit gated it at all. Mirrors
/// `require_view_invoices` in `handlers-invoice`.
async fn require_view_automation(
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

    let permissions = state
        .usecase
        .member_permissions(user.id, organization_id)
        .await?;

    if !permissions.contains(Permissions::VIEW_AUTOMATION) {
        return Err(ApiError::Forbidden);
    }

    Ok(())
}

/// Same shape as [`require_view_automation`], gating on `MANAGE_AUTOMATION`
/// instead — and, unlike `MANAGE_INVOICES` (`mestier_core::application::mod`'s
/// `default_authorizer`, action `"invoice.manage"`), gated here at the HTTP
/// boundary and nowhere else. Two reasons:
///
/// 1. The use cases this bit would otherwise gate are shared between a human
///    caller and the engine itself: `application::task_recurrence` calls
///    `start_run` and `save_workflow_version` directly, and the dispatcher
///    starts a run the moment a subscribed event fires. A `policy::require`
///    refusal inside either use case would block the engine, not a browse —
///    exactly the bug class `application::mod`'s own comment describes for
///    why `get_customer`/`get_invoice` are gated at the handler layer
///    instead (#395).
/// 2. `application::mod`'s
///    `every_registered_action_is_passed_to_policy_require_somewhere` test
///    fails on any action registered in `default_authorizer` and never
///    passed to `policy::require`. No automation use case can call one
///    without recreating reason 1, so no `"automation.manage"` action is
///    registered there at all.
async fn require_manage_automation(
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

    let permissions = state
        .usecase
        .member_permissions(user.id, organization_id)
        .await?;

    if !permissions.contains(Permissions::MANAGE_AUTOMATION) {
        return Err(ApiError::Forbidden);
    }

    Ok(())
}

pub fn router(state: &AppState) -> Router<AppState> {
    Router::new()
        .merge(catalogue::router(state))
        .merge(credential::router(state))
        .merge(workflow::router(state))
        .merge(run::router(state))
        .merge(settings::router(state))
        .layer(from_fn_with_state(state.clone(), rate_limit_middleware))
        .layer(from_fn_with_state(state.clone(), auth_middleware))
}
