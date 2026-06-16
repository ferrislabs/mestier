use common::CoreError;
use mestier_macros::transactional;

use crate::{
    Equipment, EquipmentId, OrganizationId,
    application::MestierUseCase,
    domain::equipment::{
        commands::{CreateEquipmentCommand, UpdateEquipmentCommand},
        service::EquipmentService,
    },
};

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
}
