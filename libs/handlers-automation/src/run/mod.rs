//! Runs: list with status and timestamps, read with steps, replay from a
//! step, and manual start (nested under the workflow it runs).

use axum::Router;
use axum_extra::routing::RouterExt;
use handlers::{ApiError, AppState};
use mestier_core::{OrganizationId, Run};
use uuid::Uuid;

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

/// Loads the run and checks it belongs to `organization_id` — the
/// permission gate is the caller's job first: `get_one` reads, so it calls
/// `require_view_automation`; `replay` mutates, so it calls
/// `require_manage_automation`. One loader shared by both rather than two
/// (contrast `credential::require_credential`, gated internally, since
/// every one of its callers is a write) because this one genuinely is not.
/// Mirrors `workflow::find_workflow_in_org`.
pub(crate) async fn find_run_in_org(
    state: &AppState,
    organization_id: OrganizationId,
    run_id: Uuid,
) -> Result<Run, ApiError> {
    state
        .usecase
        .find_run(organization_id, run_id)
        .await?
        .ok_or(ApiError::NotFound)
}
