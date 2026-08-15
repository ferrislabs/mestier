use std::collections::HashMap;

use chrono::{DateTime, Utc};
use common::CoreError;

use crate::{Equipment, EquipmentId, OrganizationId, TaskId};

#[cfg_attr(test, mockall::automock)]
pub trait EquipmentRepository: Send {
    fn insert(
        &mut self,
        equipment: &Equipment,
    ) -> impl Future<Output = Result<Equipment, CoreError>> + Send;

    fn find_by_id(
        &mut self,
        id: EquipmentId,
    ) -> impl Future<Output = Result<Option<Equipment>, CoreError>> + Send;

    fn list_by_organization(
        &mut self,
        organization_id: OrganizationId,
        limit: u64,
        offset: u64,
    ) -> impl Future<Output = Result<(Vec<Equipment>, u64), CoreError>> + Send;

    fn update(
        &mut self,
        equipment: &Equipment,
    ) -> impl Future<Output = Result<Equipment, CoreError>> + Send;

    fn soft_delete(
        &mut self,
        id: EquipmentId,
        deleted_at: DateTime<Utc>,
    ) -> impl Future<Output = Result<(), CoreError>> + Send;

    /// Replaces the complete set of equipment attached to `task_id`: physical
    /// delete of whatever is there, then insert of the new list. Mirrors
    /// `TaskLabelRepository::replace_links` — the `PATCH` contract treats
    /// `equipment_ids` as the full list, never a delta.
    fn replace_task_links(
        &mut self,
        task_id: TaskId,
        equipment_ids: &[EquipmentId],
    ) -> impl Future<Output = Result<(), CoreError>> + Send;

    /// Every equipment attached to each id in `task_ids`, in one grouped
    /// query — never one per task (mirrors
    /// `TaskLabelRepository::list_labels_for_tasks`'s own N+1 warning). A
    /// task with no equipment is absent from the map rather than mapped to
    /// an empty `Vec`.
    fn list_equipment_for_tasks(
        &mut self,
        task_ids: &[TaskId],
    ) -> impl Future<Output = Result<HashMap<TaskId, Vec<Equipment>>, CoreError>> + Send;
}
