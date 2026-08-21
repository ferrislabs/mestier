use chrono::{DateTime, Utc};

use crate::{
    CustomerContextId, CustomerId, EquipmentId, OrganizationId, ProjectId, QuoteId,
    domain::task::{AssigneeRef, TaskId, TaskStatus},
    domain::task_label::TaskLabelId,
};

#[derive(Debug, Clone)]
pub struct CreateTaskCommand {
    pub organization_id: OrganizationId,
    /// `None` for a root task. `Some` marks this as a subtask — its
    /// candidate parent must itself be a root (see
    /// `service::validate_parent_depth`).
    pub parent_task_id: Option<TaskId>,
    pub title: String,
    pub description: Option<String>,
    /// `None` on a subtask means "inherit the parent's window" — see
    /// `service::resolve_task_window`. A root must carry both, or neither
    /// (`service::TaskService::create_task` rejects a bare root).
    pub starts_at: Option<DateTime<Utc>>,
    pub ends_at: Option<DateTime<Utc>>,
    pub all_day: bool,
    /// Declared here, at creation, never guessed — see `Task::blocks_availability`.
    pub blocks_availability: bool,
    /// Both present or both absent — a task with a customer is a chantier.
    pub customer_id: Option<CustomerId>,
    pub customer_context_id: Option<CustomerContextId>,
    pub quote_id: Option<QuoteId>,
    /// The project this task is costed against. Independent of
    /// `parent_task_id`: a subtask can name a project its parent does not.
    pub project_id: Option<ProjectId>,
}

/// Carries a `PATCH`: every field is optional and only the ones present are
/// applied. `description`, `starts_at`, `ends_at` and `parent_task_id` are
/// themselves nullable, so they need the double option — `None` means
/// "leave unchanged", `Some(None)` means "clear it", `Some(Some(value))`
/// means "set it". `title` cannot be cleared (the column is `NOT NULL`), so
/// a single `Option` is enough for it.
#[derive(Debug, Clone)]
pub struct PatchTaskCommand {
    pub id: TaskId,
    pub parent_task_id: Option<Option<TaskId>>,
    pub title: Option<String>,
    pub description: Option<Option<String>>,
    pub starts_at: Option<Option<DateTime<Utc>>>,
    pub ends_at: Option<Option<DateTime<Utc>>>,
    pub all_day: Option<bool>,
    pub status: Option<TaskStatus>,
    pub blocks_availability: Option<bool>,
    /// The complete replacement list of assignees, or `None` to leave the
    /// current assignments untouched. Never a delta.
    pub assignees: Option<Vec<AssigneeRef>>,
    /// The complete replacement list of labels, or `None` to leave the
    /// task's current labels untouched. Same semantics as `assignees`, same
    /// reason: idempotence, and a single path for both adding and removing
    /// a label (see the planning module design doc). Applied by
    /// `MestierUseCase::patch_task` (`application/task/mod.rs`) after
    /// `TaskService::patch_task` returns — `label_ids` is carried on this
    /// command only so the field survives the trip from the HTTP layer; the
    /// `task` aggregate itself never reads it.
    pub label_ids: Option<Vec<TaskLabelId>>,
    /// The complete replacement list of equipment attached to this task, or
    /// `None` to leave it untouched. Same semantics and same reason as
    /// `label_ids` — applied by `MestierUseCase::patch_task` via
    /// `EquipmentService::replace_task_equipment` after `TaskService::patch_task`
    /// returns; the `task` aggregate itself never reads it.
    pub equipment_ids: Option<Vec<EquipmentId>>,
    /// `None` leaves the attachment alone, `Some(None)` detaches the task from
    /// its project, `Some(Some(id))` attaches it. Validated at the application
    /// seam rather than in `TaskService`, which owns no project port — see
    /// `MestierUseCase::patch_task`.
    pub project_id: Option<Option<ProjectId>>,
}

impl PatchTaskCommand {
    /// A no-op patch targeting `id`: every field left unset. Tests and
    /// callers flip on only the fields they mean to change.
    pub fn new(id: TaskId) -> Self {
        Self {
            id,
            parent_task_id: None,
            title: None,
            description: None,
            starts_at: None,
            ends_at: None,
            all_day: None,
            status: None,
            blocks_availability: None,
            assignees: None,
            label_ids: None,
            equipment_ids: None,
            project_id: None,
        }
    }
}
