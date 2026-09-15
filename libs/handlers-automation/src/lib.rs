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

/// Membership is the outer gate, `VIEW_AUTOMATION` the inner one.
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

/// Membership is the outer gate, `MANAGE_AUTOMATION` the inner one.
///
/// Do not move this into `default_authorizer` as an action: the use cases it
/// would gate are called by the engine itself, which has no human caller to
/// authorize, so a `policy::require` there refuses the engine.
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
