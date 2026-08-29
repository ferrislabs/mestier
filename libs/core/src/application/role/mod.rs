use authz::Subject;
use common::CoreError;
use mestier_macros::transactional;

use crate::{
    application::MestierUseCase,
    domain::{
        organization::OrganizationId,
        role::{
            Role, RoleId,
            commands::{CreateRoleCommand, UpdateRoleCommand},
            service::RoleService,
        },
    },
};

impl MestierUseCase {
    #[transactional(role, member, authz)]
    pub async fn create_role(&self, command: CreateRoleCommand) -> Result<Role, CoreError> {
        let mut service = RoleService::new(role_repository, member_repository, authz);
        service.create_role(command).await
    }

    #[transactional(role, member, authz)]
    pub async fn list_roles(
        &self,
        organization_id: OrganizationId,
        actor: Subject,
    ) -> Result<Vec<Role>, CoreError> {
        let mut service = RoleService::new(role_repository, member_repository, authz);
        service.list_roles(organization_id, actor).await
    }

    #[transactional(role, member, authz)]
    pub async fn update_role(&self, command: UpdateRoleCommand) -> Result<Role, CoreError> {
        let mut service = RoleService::new(role_repository, member_repository, authz);
        service.update_role(command).await
    }

    #[transactional(role, member, authz)]
    pub async fn delete_role(&self, role_id: RoleId, actor: Subject) -> Result<(), CoreError> {
        let mut service = RoleService::new(role_repository, member_repository, authz);
        service.delete_role(role_id, actor).await
    }
}
