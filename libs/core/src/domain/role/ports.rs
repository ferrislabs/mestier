use common::CoreError;

use crate::domain::{organization::OrganizationId, role::Role};

#[cfg_attr(test, mockall::automock)]
pub trait RoleRepository: Send {
    fn insert(&mut self, role: &Role) -> impl Future<Output = Result<Role, CoreError>> + Send;

    fn list_by_organization(
        &mut self,
        organization_id: OrganizationId,
    ) -> impl Future<Output = Result<Vec<Role>, CoreError>> + Send;
}
