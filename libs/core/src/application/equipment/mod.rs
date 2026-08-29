use std::collections::HashMap;

use authz::{Resource, Subject};
use common::CoreError;
use mestier_macros::transactional;

use crate::{
    Equipment, EquipmentId, OrganizationId, TaskId,
    application::{MestierUseCase, policy},
    domain::equipment::{
        commands::{CreateEquipmentCommand, UpdateEquipmentCommand},
        ports::EquipmentRepository,
        service::EquipmentService,
    },
};

mod tests;

impl MestierUseCase {
    #[transactional(equipment, role, member, authz)]
    pub async fn create_equipment(
        &self,
        command: CreateEquipmentCommand,
    ) -> Result<Equipment, CoreError> {
        // `#[transactional]` hands the repositories over immutably; the
        // policy engine needs them mutable to walk roles.
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
            "reference.manage",
            Resource::new("organization", command.organization_id.0.to_string()),
        )
        .await?;

        let mut service = EquipmentService::new(equipment_repository);
        service.create_equipment(command).await
    }

    #[transactional(equipment)]
    pub async fn get_equipment(&self, id: EquipmentId) -> Result<Equipment, CoreError> {
        let mut service = EquipmentService::new(equipment_repository);
        service.get_equipment(id).await
    }

    #[transactional(equipment)]
    pub async fn list_equipment(
        &self,
        organization_id: OrganizationId,
        limit: u64,
        offset: u64,
    ) -> Result<(Vec<Equipment>, u64), CoreError> {
        let mut service = EquipmentService::new(equipment_repository);
        service.list_equipment(organization_id, limit, offset).await
    }

    /// The equipment row is loaded first and authorization runs against
    /// *its own* `organization_id`, never one taken from the request path —
    /// a bare `/equipment/{id}` route has no organization to trust
    /// otherwise (CLAUDE.md's "bare ids derive their organization from the
    /// loaded row" rule).
    #[transactional(equipment, role, member, authz)]
    pub async fn update_equipment(
        &self,
        command: UpdateEquipmentCommand,
    ) -> Result<Equipment, CoreError> {
        let mut member_repository = member_repository;
        let mut role_repository = role_repository;

        let mut service = EquipmentService::new(equipment_repository);
        let existing = service.get_equipment(command.id).await?;

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
            "reference.manage",
            Resource::new("organization", existing.organization_id.0.to_string()),
        )
        .await?;

        service.update_equipment(command).await
    }

    /// Same "load, then authorize against the loaded row's own organization"
    /// rule as [`Self::update_equipment`] — there is no domain command to
    /// carry an `actor` for a bare-id delete, so it is threaded as its own
    /// parameter instead, the same way `remove_employee_profile` does.
    #[transactional(equipment, role, member, authz)]
    pub async fn soft_delete_equipment(
        &self,
        actor: Subject,
        id: EquipmentId,
    ) -> Result<(), CoreError> {
        let mut member_repository = member_repository;
        let mut role_repository = role_repository;

        let mut service = EquipmentService::new(equipment_repository);
        let existing = service.get_equipment(id).await?;

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
            "reference.manage",
            Resource::new("organization", existing.organization_id.0.to_string()),
        )
        .await?;

        service.soft_delete_equipment(id).await
    }

    /// Every equipment attached to each id in `task_ids`, in one grouped
    /// query — never one per task. Feeds `equipment` on every `TaskResponse`
    /// surface (`handlers-planning/src/task/{get_one,list,update}.rs`),
    /// mirroring `list_task_labels_for_tasks`.
    #[transactional(equipment)]
    pub async fn list_equipment_for_tasks(
        &self,
        task_ids: Vec<TaskId>,
    ) -> Result<HashMap<TaskId, Vec<Equipment>>, CoreError> {
        let mut equipment_repository = equipment_repository;
        equipment_repository
            .list_equipment_for_tasks(&task_ids)
            .await
    }
}
