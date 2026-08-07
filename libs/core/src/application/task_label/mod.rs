use common::CoreError;
use mestier_macros::transactional;

use crate::{
    OrganizationId, TaskLabel, TaskLabelId,
    application::MestierUseCase,
    domain::task_label::{
        commands::{CreateTaskLabelCommand, UpdateTaskLabelCommand},
        service::TaskLabelService,
    },
};

mod tests;

impl MestierUseCase {
    #[transactional(task_label)]
    pub async fn create_task_label(
        &self,
        command: CreateTaskLabelCommand,
    ) -> Result<TaskLabel, CoreError> {
        let mut service = TaskLabelService::new(task_label_repository);
        service.create_task_label(command).await
    }

    #[transactional(task_label)]
    pub async fn get_task_label(&self, id: TaskLabelId) -> Result<TaskLabel, CoreError> {
        let mut service = TaskLabelService::new(task_label_repository);
        service.get_task_label(id).await
    }

    #[transactional(task_label)]
    pub async fn list_task_labels(
        &self,
        organization_id: OrganizationId,
    ) -> Result<Vec<TaskLabel>, CoreError> {
        let mut service = TaskLabelService::new(task_label_repository);
        service.list_task_labels(organization_id).await
    }

    #[transactional(task_label)]
    pub async fn update_task_label(
        &self,
        command: UpdateTaskLabelCommand,
    ) -> Result<TaskLabel, CoreError> {
        let mut service = TaskLabelService::new(task_label_repository);
        service.update_task_label(command).await
    }

    #[transactional(task_label)]
    pub async fn delete_task_label(&self, id: TaskLabelId) -> Result<(), CoreError> {
        let mut service = TaskLabelService::new(task_label_repository);
        service.delete_task_label(id).await
    }
}
