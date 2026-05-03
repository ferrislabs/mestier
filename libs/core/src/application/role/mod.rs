use common::CoreError;
use mestier_macros::transactional;

use crate::{
    application::MestierUseCase,
    domain::{
        organization::OrganizationId,
        role::{Role, commands::CreateRoleCommand, service::RoleService},
    },
};

impl MestierUseCase {
    #[transactional(role)]
    pub async fn create_role(&self, command: CreateRoleCommand) -> Result<Role, CoreError> {
        let mut service = RoleService::new(role_repository);
        service.create_role(command).await
    }

    #[transactional(role)]
    pub async fn list_roles(
        &self,
        organization_id: OrganizationId,
    ) -> Result<Vec<Role>, CoreError> {
        let mut service = RoleService::new(role_repository);
        service.list_roles(organization_id).await
    }
}
