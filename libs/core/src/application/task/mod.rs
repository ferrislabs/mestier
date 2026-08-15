use std::collections::HashMap;

use common::CoreError;
use mestier_macros::transactional;

use crate::{
    OrganizationId, Task, TaskId,
    application::MestierUseCase,
    domain::equipment::service::EquipmentService,
    domain::task::{
        commands::{CreateTaskCommand, PatchTaskCommand},
        service::TaskService,
    },
    domain::task_label::service::TaskLabelService,
};

mod tests;

impl MestierUseCase {
    #[transactional(task, member)]
    pub async fn create_task(&self, command: CreateTaskCommand) -> Result<Task, CoreError> {
        let mut service = TaskService::new(task_repository, member_repository);
        service.create_task(command).await
    }

    #[transactional(task, member)]
    pub async fn get_task(&self, id: TaskId) -> Result<Task, CoreError> {
        let mut service = TaskService::new(task_repository, member_repository);
        service.get_task(id).await
    }

    /// Lists a page of `organization_id`'s tasks — every root when
    /// `parent_task_id` is `None`, or a specific task's children otherwise —
    /// together with each returned task's own child count (see
    /// `TaskService::list_tasks`: computed in one grouped query, never one
    /// per task).
    #[transactional(task, member)]
    pub async fn list_tasks(
        &self,
        organization_id: OrganizationId,
        parent_task_id: Option<TaskId>,
        limit: u64,
        offset: u64,
    ) -> Result<(Vec<Task>, HashMap<TaskId, i64>, u64), CoreError> {
        let mut service = TaskService::new(task_repository, member_repository);
        service
            .list_tasks(organization_id, parent_task_id, limit, offset)
            .await
    }

    /// Reparents, reschedules and reassigns a task in one transaction:
    /// either every write here (the parent/schedule/status/title/description
    /// edits, the `blocks_availability` flag, the full assignment
    /// replacement, any on-the-fly employee record, the full label
    /// replacement, and — the piece this workstream adds — the full
    /// equipment replacement) lands together, or the whole `PATCH` rolls
    /// back.
    ///
    /// Label and equipment handling live here rather than inside
    /// `TaskService::patch_task` on purpose: `task`, `task_label` and
    /// `equipment` are deliberately separate aggregates (see the planning
    /// module design doc — T2 and T3 must not collide on the same domain
    /// files), so composing `TaskLabelRepository`/`EquipmentRepository` at
    /// this thin, already-transactional seam avoids adding a dependency from
    /// `task`'s own domain service onto a sibling aggregate's port.
    #[transactional(task, member, task_label, equipment)]
    pub async fn patch_task(&self, command: PatchTaskCommand) -> Result<Task, CoreError> {
        let label_ids = command.label_ids.clone();
        let equipment_ids = command.equipment_ids.clone();

        let mut service = TaskService::new(task_repository, member_repository);
        let task = service.patch_task(command).await?;

        if let Some(label_ids) = label_ids {
            let mut label_service = TaskLabelService::new(task_label_repository);
            label_service
                .replace_task_labels(task.organization_id, task.id, label_ids)
                .await?;
        }

        if let Some(equipment_ids) = equipment_ids {
            let mut equipment_service = EquipmentService::new(equipment_repository);
            equipment_service
                .replace_task_equipment(task.organization_id, task.id, equipment_ids)
                .await?;
        }

        Ok(task)
    }

    #[transactional(task, member)]
    pub async fn soft_delete_task(&self, id: TaskId) -> Result<(), CoreError> {
        let mut service = TaskService::new(task_repository, member_repository);
        service.soft_delete_task(id).await
    }
}
