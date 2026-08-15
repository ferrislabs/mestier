use std::collections::HashMap;

use common::CoreError;
use mestier_macros::transactional;

use crate::{
    Equipment, EquipmentId, OrganizationId, TaskId,
    application::MestierUseCase,
    domain::equipment::{
        commands::{CreateEquipmentCommand, UpdateEquipmentCommand},
        ports::EquipmentRepository,
        service::EquipmentService,
    },
};

mod tests;

impl MestierUseCase {
    #[transactional(equipment)]
    pub async fn create_equipment(
        &self,
        command: CreateEquipmentCommand,
    ) -> Result<Equipment, CoreError> {
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

    #[transactional(equipment)]
    pub async fn update_equipment(
        &self,
        command: UpdateEquipmentCommand,
    ) -> Result<Equipment, CoreError> {
        let mut service = EquipmentService::new(equipment_repository);
        service.update_equipment(command).await
    }

    #[transactional(equipment)]
    pub async fn soft_delete_equipment(&self, id: EquipmentId) -> Result<(), CoreError> {
        let mut service = EquipmentService::new(equipment_repository);
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
