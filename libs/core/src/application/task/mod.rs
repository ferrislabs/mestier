use std::collections::HashMap;

use authz::Resource;
use common::CoreError;
use mestier_macros::transactional;

use crate::{
    AssigneeRef, OrganizationId, Task, TaskId,
    application::{MestierUseCase, policy},
    domain::equipment::service::EquipmentService,
    domain::project::service::ProjectService,
    domain::task::{
        DeleteScope,
        commands::{CreateTaskCommand, PatchTaskCommand},
        ports::TaskRepository,
        service::TaskService,
    },
    domain::task_label::service::TaskLabelService,
};

mod tests;

impl MestierUseCase {
    #[transactional(task, member, project, role, authz)]
    pub async fn create_task(&self, command: CreateTaskCommand) -> Result<Task, CoreError> {
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

        if let Some(project_id) = command.project_id {
            let mut project_service = ProjectService::new(project_repository);
            let project = project_service.get_project(project_id).await?;
            if project.organization_id != command.organization_id {
                return Err(CoreError::NotFound);
            }
        }

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
    #[transactional(task, member, task_label, equipment, project, role, authz)]
    pub async fn patch_task(&self, command: PatchTaskCommand) -> Result<Task, CoreError> {
        let label_ids = command.label_ids.clone();
        let equipment_ids = command.equipment_ids.clone();

        let mut task_repository = task_repository;
        let mut member_repository = member_repository;
        let mut role_repository = role_repository;

        // A bare id derives its organization from the loaded row, never from
        // the command — see the module doc. Loaded before `TaskService` takes
        // ownership of `task_repository`/`member_repository`, so this can
        // still borrow them for the authorization check below.
        let existing = task_repository
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

        // Resolved before anything is written. The composite foreign key on
        // `(project_id, org_id)` already makes a cross-organization attachment
        // impossible, but it fails as a plain database error, which surfaces as
        // a 500. One indexed read inside the transaction buys the 404 the
        // caller deserves. Mirrors `replace_task_equipment`'s own rule: an
        // unknown id, or one from another organization, is `NotFound`.
        let mut service = TaskService::new(task_repository, member_repository);

        if let Some(Some(project_id)) = command.project_id {
            let mut project_service = ProjectService::new(project_repository);
            if project_service
                .get_project(project_id)
                .await?
                .organization_id
                != existing.organization_id
            {
                return Err(CoreError::NotFound);
            }
        }

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

    #[transactional(task, member, role, authz)]
    pub async fn soft_delete_task(
        &self,
        actor: authz::Subject,
        id: TaskId,
    ) -> Result<(), CoreError> {
        let mut task_repository = task_repository;
        let mut member_repository = member_repository;
        let mut role_repository = role_repository;

        let existing = task_repository
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

        let mut service = TaskService::new(task_repository, member_repository);
        service.soft_delete_task(id).await
    }

    /// The scope-aware `DELETE`: `ThisOccurrence` behaves exactly like
    /// [`Self::soft_delete_task`]; `ThisAndFollowing` also removes every
    /// later occurrence in the same series — see
    /// `TaskService::soft_delete_occurrence`.
    #[transactional(task, member, role, authz)]
    pub async fn soft_delete_task_occurrence(
        &self,
        actor: authz::Subject,
        id: TaskId,
        scope: DeleteScope,
    ) -> Result<(), CoreError> {
        let mut task_repository = task_repository;
        let mut member_repository = member_repository;
        let mut role_repository = role_repository;

        let existing = task_repository
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

        let mut service = TaskService::new(task_repository, member_repository);
        service.soft_delete_occurrence(id, scope).await
    }

    /// Assigns `assignees` to every task in `task_ids` in one transaction —
    /// all or nothing, never one HTTP call per task. See
    /// `TaskService::bulk_assign_tasks`'s own doc for the failure contract:
    /// the first missing or foreign task fails the whole call, rolling back
    /// any earlier task's write in the same batch.
    #[transactional(task, member, role, authz)]
    pub async fn bulk_assign_tasks(
        &self,
        actor: authz::Subject,
        organization_id: OrganizationId,
        task_ids: Vec<TaskId>,
        assignees: Vec<AssigneeRef>,
    ) -> Result<Vec<Task>, CoreError> {
        let mut member_repository = member_repository;
        let mut role_repository = role_repository;

        let actor = policy::enrich_for_organization(
            actor,
            organization_id,
            &mut member_repository,
            &mut role_repository,
        )
        .await?;
        policy::require(
            &authz,
            &actor,
            "planning.manage",
            Resource::new("organization", organization_id.0.to_string()),
        )
        .await?;

        let mut service = TaskService::new(task_repository, member_repository);
        service
            .bulk_assign_tasks(organization_id, task_ids, assignees)
            .await
    }
}
