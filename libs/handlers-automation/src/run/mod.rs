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

/// Loads a run of `organization_id`. Gates nothing: readers and writers
/// share it, so the caller applies its own gate first.
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
