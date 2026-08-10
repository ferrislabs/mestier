//! Runs: list with status and timestamps, read with steps, replay from a
//! step, and manual start (nested under the workflow it runs).

use auth::Identity;
use axum::Router;
use axum_extra::routing::RouterExt;
use handlers::{ApiError, AppState};
use mestier_core::{OrganizationId, Run};
use uuid::Uuid;

use crate::require_org_membership;

pub mod get_one;
pub mod list;
pub mod replay;
pub mod start;

pub fn router(_state: &AppState) -> Router<AppState> {
    Router::new()
        .typed_get(list::handler)
        .typed_get(get_one::handler)
        .typed_post(replay::handler)
        .typed_post(start::handler)
}

/// Loads the run and checks both that the caller belongs to
/// `organization_id` and that the run actually belongs to it — mirrors
/// `credential::require_credential` and `workflow::require_workflow`.
pub(crate) async fn require_run(
    state: &AppState,
    identity: &Identity,
    organization_id: OrganizationId,
    run_id: Uuid,
) -> Result<Run, ApiError> {
    require_org_membership(state, identity, organization_id).await?;

    state
        .usecase
        .find_run(organization_id, run_id)
        .await?
        .ok_or(ApiError::NotFound)
}
