use common::CoreError;
use mestier_macros::transactional;

use crate::{
    OrganizationId, TaskRecurrence, TaskRecurrenceId,
    application::MestierUseCase,
    domain::task_recurrence::{
        commands::CreateTaskRecurrenceCommand,
        service::{DEFAULT_HORIZON_DAYS, TaskRecurrenceService},
    },
};

#[cfg(test)]
mod tests;

impl MestierUseCase {
    /// Creates a recurrence and materializes its occurrences up to the
    /// default horizon — see `TaskRecurrenceService::create_recurrence`.
    /// #293 replaces the hard-coded `DEFAULT_HORIZON_DAYS` with an
    /// organization-level override; every occurrence is materialized inside
    /// this one transaction either way, so a validation failure partway
    /// through (an unknown assignee, say) leaves nothing behind.
    #[transactional(task_recurrence, task, member)]
    pub async fn create_task_recurrence(
        &self,
        command: CreateTaskRecurrenceCommand,
    ) -> Result<TaskRecurrence, CoreError> {
        let mut service = TaskRecurrenceService::new(
            task_recurrence_repository,
            task_repository,
            member_repository,
        );
        service
            .create_recurrence(command, DEFAULT_HORIZON_DAYS)
            .await
    }

    #[transactional(task_recurrence, task, member)]
    pub async fn get_task_recurrence(
        &self,
        id: TaskRecurrenceId,
    ) -> Result<TaskRecurrence, CoreError> {
        let mut service = TaskRecurrenceService::new(
            task_recurrence_repository,
            task_repository,
            member_repository,
        );
        service.get_recurrence(id).await
    }

    #[transactional(task_recurrence, task, member)]
    pub async fn list_task_recurrences(
        &self,
        organization_id: OrganizationId,
    ) -> Result<Vec<TaskRecurrence>, CoreError> {
        let mut service = TaskRecurrenceService::new(
            task_recurrence_repository,
            task_repository,
            member_repository,
        );
        service.list_recurrences(organization_id).await
    }
}
