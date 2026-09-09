use authz::Resource;
use common::CoreError;
use mestier_macros::transactional;

use crate::{
    CustomUnit, OrganizationId,
    application::{MestierUseCase, policy},
    domain::custom_unit::{commands::CreateCustomUnitCommand, service::CustomUnitService},
};

impl MestierUseCase {
    #[transactional(custom_unit, role, member, authz)]
    pub async fn create_custom_unit(
        &self,
        command: CreateCustomUnitCommand,
    ) -> Result<CustomUnit, CoreError> {
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

        let mut service = CustomUnitService::new(custom_unit_repository);
        service.create_custom_unit(command).await
    }

    #[transactional(custom_unit)]
    pub async fn list_custom_units(
        &self,
        organization_id: OrganizationId,
    ) -> Result<Vec<CustomUnit>, CoreError> {
        let mut service = CustomUnitService::new(custom_unit_repository);
        service.list_custom_units(organization_id).await
    }
}
