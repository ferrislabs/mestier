//! Task labels — tags that absorb what used to be a task's "nature"
//! (meeting, trip, training...). Not a column, not an enum: CRUD lives here;
//! assignment to a task travels through `PATCH /tasks`'s `label_ids` (see
//! `crate::task::update`), and `GET /planning`'s `task` entries expose the
//! attached labels (see `crate::planning::get_planning`).
//!
//! Owns its own leaf paths, like `crate::task`, since `organization_id` and
//! `label_id` are both part of every single-item route.

use auth::Identity;
use axum::Router;
use axum_extra::routing::{RouterExt, TypedPath};
use chrono::{DateTime, Utc};
use handlers::{ApiError, AppState};
use mestier_core::{OrganizationId, TaskLabel, TaskLabelId};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::require_org_membership;

pub mod create;
pub mod delete;
pub mod list;
pub mod update;

/// This aggregate's routes, unlayered — the crate-level `router` in
/// `lib.rs` merges every aggregate submodule's router before applying the
/// shared rate-limit/auth middleware once, rather than each submodule
/// layering its own.
pub fn router(_state: &AppState) -> Router<AppState> {
    Router::new()
        .typed_get(list::handler)
        .typed_post(create::handler)
        .typed_patch(update::handler)
        .typed_delete(delete::handler)
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/organizations/{organization_id}/task-labels")]
pub struct TaskLabelsPath {
    pub organization_id: OrganizationId,
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/organizations/{organization_id}/task-labels/{label_id}")]
pub struct TaskLabelPath {
    pub organization_id: OrganizationId,
    pub label_id: TaskLabelId,
}

/// Loads the label and checks both that the caller belongs to
/// `organization_id` and that the label actually belongs to it —
/// `organization_id` is part of every route, so a mismatch (a real
/// `label_id` from a different organization) is treated the same as "does
/// not exist" rather than leaking cross-tenant existence via 403. Mirrors
/// `crate::task::require_task`.
pub(crate) async fn require_task_label(
    state: &AppState,
    identity: &Identity,
    organization_id: OrganizationId,
    label_id: TaskLabelId,
) -> Result<TaskLabel, ApiError> {
    require_org_membership(state, identity, organization_id).await?;

    let label = state.usecase.get_task_label(label_id).await?;
    if label.organization_id != organization_id {
        return Err(ApiError::NotFound);
    }

    Ok(label)
}

/// Shared by `create`/`list`/`update`, and reused from `crate::planning`'s
/// `PlanningEntryResponse::Task::labels`.
#[derive(Debug, Clone, PartialEq, Serialize, ToSchema)]
pub struct TaskLabelResponse {
    pub id: TaskLabelId,
    pub organization_id: OrganizationId,
    pub name: String,
    pub color: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<TaskLabel> for TaskLabelResponse {
    fn from(value: TaskLabel) -> Self {
        Self {
            id: value.id,
            organization_id: value.organization_id,
            name: value.name,
            color: value.color,
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}
