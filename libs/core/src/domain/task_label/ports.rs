use std::collections::HashMap;

use common::CoreError;

use crate::{OrganizationId, TaskId, TaskLabel, TaskLabelId};

#[cfg_attr(any(test, feature = "mock"), mockall::automock)]
pub trait TaskLabelRepository: Send {
    fn insert(
        &mut self,
        label: &TaskLabel,
    ) -> impl Future<Output = Result<TaskLabel, CoreError>> + Send;

    fn find_by_id(
        &mut self,
        id: TaskLabelId,
    ) -> impl Future<Output = Result<Option<TaskLabel>, CoreError>> + Send;

    fn list_by_organization(
        &mut self,
        organization_id: OrganizationId,
    ) -> impl Future<Output = Result<Vec<TaskLabel>, CoreError>> + Send;

    fn update(
        &mut self,
        label: &TaskLabel,
    ) -> impl Future<Output = Result<TaskLabel, CoreError>> + Send;

    /// Physical deletion — `task_label_links.label_id` cascades, so removing
    /// a label removes it from every task that carried it in the same
    /// statement.
    fn delete(&mut self, id: TaskLabelId) -> impl Future<Output = Result<(), CoreError>> + Send;

    /// Replaces the complete label set attached to `task_id`: physical
    /// delete of whatever is there, then insert of the new list. Mirrors
    /// `TaskRepository`'s own assignment replacement — the `PATCH` contract
    /// treats `label_ids` as the full list, never a delta.
    fn replace_links(
        &mut self,
        task_id: TaskId,
        label_ids: &[TaskLabelId],
    ) -> impl Future<Output = Result<(), CoreError>> + Send;

    /// Every label attached to each id in `task_ids`, in one grouped query
    /// — never one per task (mirrors `TaskRepository::count_children`'s N+1
    /// warning). A task with no labels is absent from the map rather than
    /// mapped to an empty `Vec`.
    fn list_labels_for_tasks(
        &mut self,
        task_ids: &[TaskId],
    ) -> impl Future<Output = Result<HashMap<TaskId, Vec<TaskLabel>>, CoreError>> + Send;
}
