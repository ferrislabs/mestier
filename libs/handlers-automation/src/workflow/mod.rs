//! Workflows: list, create, read (with its current version), save a new
//! version, enable/disable and rename (`update`), delete, and read/set the
//! event(s) that trigger it (`trigger`, #225).

use axum::Router;
use axum_extra::routing::RouterExt;
use handlers::{ApiError, AppState};
use mestier_core::{OrganizationId, Workflow};
use uuid::Uuid;

pub mod create;
pub mod delete;
pub mod get_one;
pub mod list;
pub mod save_version;
pub mod trigger;
pub mod update;

pub fn router(_state: &AppState) -> Router<AppState> {
    Router::new()
        .typed_get(list::handler)
        .typed_post(create::handler)
        .typed_get(get_one::handler)
        .typed_patch(update::handler)
        .typed_delete(delete::handler)
        .typed_put(save_version::handler)
        .typed_get(trigger::get_trigger)
        .typed_put(trigger::set_trigger)
}

/// Loads the workflow and checks it belongs to `organization_id` — the
/// permission gate is the caller's job first, since a workflow is read by
/// some routes (`get_one`, `trigger::get_trigger`) and written by others
/// (`create` and `list` need no single workflow, but `delete`,
/// `save_version`, `trigger::set_trigger`, `update`, and `run::start`
/// — which starts a run on it — all do). One loader shared by both
/// directions rather than two, the same call `run::find_run_in_org` makes
/// for the same reason. Mirrors `handlers-planning::task::require_task`.
pub(crate) async fn find_workflow_in_org(
    state: &AppState,
    organization_id: OrganizationId,
    workflow_id: Uuid,
) -> Result<Workflow, ApiError> {
    state
        .usecase
        .find_workflow(organization_id, workflow_id)
        .await?
        .ok_or(ApiError::NotFound)
}
