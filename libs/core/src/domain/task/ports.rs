use std::collections::HashMap;

use chrono::{DateTime, NaiveDate, Utc};
use common::CoreError;

use crate::{MemberId, OrganizationId, Task, TaskId, TaskRecurrenceId};

#[cfg_attr(any(test, feature = "mock"), mockall::automock)]
pub trait TaskRepository: Send {
    fn insert(&mut self, task: &Task) -> impl Future<Output = Result<Task, CoreError>> + Send;

    /// Inserts a materialized recurrence occurrence, or does nothing when
    /// its `(recurrence_id, occurrence_date)` is already taken by a
    /// non-deleted row — the arbiter is the partial unique index
    /// `uq_tasks_recurrence_occurrence`. Returns whether a row was actually
    /// inserted, which is what makes a horizon extension idempotent: a retry
    /// or a double claim finds every date already filled and inserts
    /// nothing, rather than erroring or double-booking.
    ///
    /// `task.id` must already be set by the caller (unlike [`Self::insert`],
    /// there is no ambiguity to resolve: on conflict, the id offered is
    /// simply discarded along with the rest of the row).
    fn insert_occurrence_if_absent(
        &mut self,
        task: &Task,
    ) -> impl Future<Output = Result<bool, CoreError>> + Send;

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

    /// The tasks assigned to one member that overlap a day, for the field app.
    ///
    /// Deliberately narrow rather than filtering the organization's planning:
    /// an employee's phone must not receive everyone else's schedule, and a
    /// day's worth of one person's jobs is a handful of rows.
    ///
    /// A task with no window is included, since an undated job still has to be
    /// doable; one is excluded only when its window is entirely elsewhere.
    fn list_for_assignee_on(
        &mut self,
        organization_id: OrganizationId,
        member_id: MemberId,
        day_starts_at: DateTime<Utc>,
        day_ends_at: DateTime<Utc>,
    ) -> impl Future<Output = Result<Vec<Task>, CoreError>> + Send;

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

    /// Soft-deletes every non-deleted occurrence of `recurrence_id` whose
    /// `occurrence_date` is `>= from` — the scope-aware half of a `DELETE`
    /// asking for "this occurrence and every later one", and what
    /// `TaskRecurrenceService::delete_recurrence` uses to remove a deleted
    /// series' future occurrences while leaving its past ones alone.
    /// Returns how many rows were affected.
    fn soft_delete_recurrence_occurrences_from(
        &mut self,
        recurrence_id: TaskRecurrenceId,
        from: NaiveDate,
        deleted_at: DateTime<Utc>,
    ) -> impl Future<Output = Result<u64, CoreError>> + Send;
}
