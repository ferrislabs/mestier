use std::collections::HashMap;

use common::CoreError;
use mestier_macros::transactional;

use crate::{
    Employee, OrganizationId, Task, TaskId,
    application::MestierUseCase,
    domain::task::{
        commands::{CreateTaskCommand, PatchTaskCommand},
        service::TaskService,
    },
};

mod tests;

impl MestierUseCase {
    #[transactional(task, employee, user, member)]
    pub async fn create_task(&self, command: CreateTaskCommand) -> Result<Task, CoreError> {
        let mut service = TaskService::new(
            task_repository,
            employee_repository,
            user_repository,
            member_repository,
        );
        service.create_task(command).await
    }

    #[transactional(task, employee, user, member)]
    pub async fn get_task(&self, id: TaskId) -> Result<Task, CoreError> {
        let mut service = TaskService::new(
            task_repository,
            employee_repository,
            user_repository,
            member_repository,
        );
        service.get_task(id).await
    }

    /// Lists a page of `organization_id`'s tasks — every root when
    /// `parent_task_id` is `None`, or a specific task's children otherwise —
    /// together with each returned task's own child count (see
    /// `TaskService::list_tasks`: computed in one grouped query, never one
    /// per task).
    #[transactional(task, employee, user, member)]
    pub async fn list_tasks(
        &self,
        organization_id: OrganizationId,
        parent_task_id: Option<TaskId>,
        limit: u64,
        offset: u64,
    ) -> Result<(Vec<Task>, HashMap<TaskId, i64>, u64), CoreError> {
        let mut service = TaskService::new(
            task_repository,
            employee_repository,
            user_repository,
            member_repository,
        );
        service
            .list_tasks(organization_id, parent_task_id, limit, offset)
            .await
    }

    /// Reparents, reschedules and reassigns a task in one transaction:
    /// either every write here (the parent/schedule/status/title/description
    /// edits, the `blocks_availability` flag, the full assignment
    /// replacement, and any on-the-fly employee record) lands together, or
    /// the whole `PATCH` rolls back.
    #[transactional(task, employee, user, member)]
    pub async fn patch_task(
        &self,
        command: PatchTaskCommand,
    ) -> Result<(Task, Vec<Employee>), CoreError> {
        let mut service = TaskService::new(
            task_repository,
            employee_repository,
            user_repository,
            member_repository,
        );
        service.patch_task(command).await
    }

    #[transactional(task, employee, user, member)]
    pub async fn soft_delete_task(&self, id: TaskId) -> Result<(), CoreError> {
        let mut service = TaskService::new(
            task_repository,
            employee_repository,
            user_repository,
            member_repository,
        );
        service.soft_delete_task(id).await
    }
}
