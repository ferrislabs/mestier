use std::collections::HashMap;

use authz::Resource;
use common::CoreError;
use mestier_macros::transactional;

use crate::{
    OrganizationId, TaskId, TaskLabel, TaskLabelId,
    application::{MestierUseCase, policy},
    domain::task_label::{
        commands::{CreateTaskLabelCommand, UpdateTaskLabelCommand},
        ports::TaskLabelRepository,
        service::TaskLabelService,
    },
};

mod tests;

impl MestierUseCase {
    #[transactional(task_label, role, member, authz)]
    pub async fn create_task_label(
        &self,
        command: CreateTaskLabelCommand,
    ) -> Result<TaskLabel, CoreError> {
        let mut member_repository = member_repository;
        let mut role_repository = role_repository;

        let actor = policy::enrich_for_organization(
            command.actor.clone(),
            command.organization_id,
            &mut member_repository,
            &mut role_repository,
        )
        .await?;
        policy::require(
            &authz,
            &actor,
            "planning.manage",
            Resource::new("organization", command.organization_id.0.to_string()),
        )
        .await?;

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

    #[transactional(task_label, role, member, authz)]
    pub async fn update_task_label(
        &self,
        command: UpdateTaskLabelCommand,
    ) -> Result<TaskLabel, CoreError> {
        let mut task_label_repository = task_label_repository;
        let mut member_repository = member_repository;
        let mut role_repository = role_repository;

        // A bare id derives its organization from the loaded row, never from
        // the command.
        let existing = task_label_repository
            .find_by_id(command.id)
            .await?
            .ok_or(CoreError::NotFound)?;

        let actor = policy::enrich_for_organization(
            command.actor.clone(),
            existing.organization_id,
            &mut member_repository,
            &mut role_repository,
        )
        .await?;
        policy::require(
            &authz,
            &actor,
            "planning.manage",
            Resource::new("organization", existing.organization_id.0.to_string()),
        )
        .await?;

        let mut service = TaskLabelService::new(task_label_repository);
        service.update_task_label(command).await
    }

    #[transactional(task_label, role, member, authz)]
    pub async fn delete_task_label(
        &self,
        actor: authz::Subject,
        id: TaskLabelId,
    ) -> Result<(), CoreError> {
        let mut task_label_repository = task_label_repository;
        let mut member_repository = member_repository;
        let mut role_repository = role_repository;

        let existing = task_label_repository
            .find_by_id(id)
            .await?
            .ok_or(CoreError::NotFound)?;

        let actor = policy::enrich_for_organization(
            actor,
            existing.organization_id,
            &mut member_repository,
            &mut role_repository,
        )
        .await?;
        policy::require(
            &authz,
            &actor,
            "planning.manage",
            Resource::new("organization", existing.organization_id.0.to_string()),
        )
        .await?;

        let mut service = TaskLabelService::new(task_label_repository);
        service.delete_task_label(id).await
    }

    /// Every label attached to each id in `task_ids`, in one grouped query
    /// — never one per task. Feeds `labels` on all four `TaskResponse`
    /// surfaces (`handlers-planning/src/task/{create,get_one,list,update}.rs`):
    /// `GET /tasks`'s list handler calls this once for the whole page, not
    /// once per row (see `TaskLabelRepository::list_labels_for_tasks`'s own
    /// N+1 warning, mirrored from `TaskRepository::count_children`).
    #[transactional(task_label)]
    pub async fn list_task_labels_for_tasks(
        &self,
        task_ids: Vec<TaskId>,
    ) -> Result<HashMap<TaskId, Vec<TaskLabel>>, CoreError> {
        let mut task_label_repository = task_label_repository;
        task_label_repository.list_labels_for_tasks(&task_ids).await
    }
}
