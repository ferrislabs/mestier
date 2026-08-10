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
use mestier_core::OrganizationId;

pub mod catalogue;
pub mod credential;
pub mod paths;
pub mod response;
pub mod run;
pub mod settings;
pub mod workflow;

pub const TAG: &str = "automation";

/// Rejects an identity that does not belong to the organization.
///
/// Identical in shape to every other handler crate's own copy (see
/// `handlers-planning::require_org_membership`'s doc comment for why this is
/// honest duplication rather than a shared module): each crate is a separate
/// HTTP adapter, and the repository already duplicates the same query.
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
        .merge(catalogue::router(state))
        .merge(credential::router(state))
        .merge(workflow::router(state))
        .merge(run::router(state))
        .merge(settings::router(state))
        .layer(from_fn_with_state(state.clone(), rate_limit_middleware))
        .layer(from_fn_with_state(state.clone(), auth_middleware))
}
