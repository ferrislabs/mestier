//! Task recurrences: the rule that repeats a task, exposed alongside the
//! `tasks` routes it materializes into.
//!
//! `POST`/`GET` are organization-scoped (`organization_id` in the path,
//! matching `TasksPath`); `PATCH`/`DELETE` are bare-id routes — the caller
//! addresses a recurrence directly, and its organization is derived from the
//! loaded row rather than trusted from a path segment, mirroring
//! `require_member_target` in `crate::lib` (see its own doc for why a bare id
//! never takes its organization from the caller).

use auth::Identity;
use axum::Router;
use axum_extra::routing::{RouterExt, TypedPath};
use handlers::{ApiError, AppState};
use mestier_core::{OrganizationId, TaskRecurrence, TaskRecurrenceId};
use serde::Deserialize;

use crate::require_org_membership;

pub mod create;
pub mod delete;
pub mod list;
pub mod update;

pub fn router(_state: &AppState) -> Router<AppState> {
    Router::new()
        .typed_post(create::handler)
        .typed_get(list::handler)
        .typed_patch(update::handler)
        .typed_delete(delete::handler)
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/organizations/{organization_id}/task-recurrences")]
pub struct TaskRecurrencesPath {
    pub organization_id: OrganizationId,
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/task-recurrences/{task_recurrence_id}")]
pub struct TaskRecurrencePath {
    pub task_recurrence_id: TaskRecurrenceId,
}

/// Loads the recurrence and checks the caller belongs to *its* organization
/// — never an organization taken from the path, since bare-id routes have
/// none. A stranger's id reads back `NotFound` (from the load) rather than
/// `Forbidden`, so existence never leaks across tenants.
pub(crate) async fn require_recurrence_target(
    state: &AppState,
    identity: &Identity,
    id: TaskRecurrenceId,
) -> Result<TaskRecurrence, ApiError> {
    let recurrence = state.usecase.get_task_recurrence(id).await?;
    require_org_membership(state, identity, recurrence.organization_id).await?;
    Ok(recurrence)
}
