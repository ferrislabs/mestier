use std::collections::HashMap;

use chrono::{DateTime, Utc};
use common::CoreError;

use crate::{OrganizationId, Task, TaskId};

#[cfg_attr(test, mockall::automock)]
pub trait TaskRepository: Send {
    fn insert(&mut self, task: &Task) -> impl Future<Output = Result<Task, CoreError>> + Send;

    fn find_by_id(
        &mut self,
        id: TaskId,
    ) -> impl Future<Output = Result<Option<Task>, CoreError>> + Send;

    /// Every task of `organization_id`, optionally scoped to the children of
    /// `parent_task_id` (`None` lists roots). Never returns a child's own
    /// children — the read model asks for those with a second call; see
    /// [`Self::count_children`] for how the list view learns each root's
    /// child count without loading them.
    fn list_by_organization(
        &mut self,
        organization_id: OrganizationId,
        parent_task_id: Option<TaskId>,
        limit: u64,
        offset: u64,
    ) -> impl Future<Output = Result<(Vec<Task>, u64), CoreError>> + Send;

    /// The number of direct children of each id in `task_ids`, in one
    /// grouped query — never one query per task (see the planning module
    /// design doc's N+1 warning and `GET /tasks`'s contract: it reports each
    /// root's child count without loading the hierarchy). An id with no
    /// children is absent from the map rather than mapped to `0`.
    fn count_children(
        &mut self,
        task_ids: &[TaskId],
    ) -> impl Future<Output = Result<HashMap<TaskId, i64>, CoreError>> + Send;

    /// Persists both the task's own fields and its complete `assignments`
    /// list. The infra adapter replaces assignments as a whole (physical
    /// delete then insert) rather than diffing — matching the `PATCH`
    /// contract, where `assignees` is never a delta.
    fn update(&mut self, task: &Task) -> impl Future<Output = Result<Task, CoreError>> + Send;

    fn soft_delete(
        &mut self,
        id: TaskId,
        deleted_at: DateTime<Utc>,
    ) -> impl Future<Output = Result<(), CoreError>> + Send;
}
