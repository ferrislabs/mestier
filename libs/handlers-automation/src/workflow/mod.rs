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

/// Loads a workflow of `organization_id`. Gates nothing: readers and writers
/// share it, so the caller applies its own gate first.
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
