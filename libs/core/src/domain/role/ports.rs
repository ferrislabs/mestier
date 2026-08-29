use common::CoreError;

use crate::domain::{
    organization::OrganizationId,
    role::{Role, RoleId},
};

#[cfg_attr(any(test, feature = "mock"), mockall::automock)]
pub trait RoleRepository: Send {
    fn insert(&mut self, role: &Role) -> impl Future<Output = Result<Role, CoreError>> + Send;

    fn find_by_id(
        &mut self,
        id: RoleId,
    ) -> impl Future<Output = Result<Option<Role>, CoreError>> + Send;

    fn list_by_organization(
        &mut self,
        organization_id: OrganizationId,
    ) -> impl Future<Output = Result<Vec<Role>, CoreError>> + Send;

    fn update(&mut self, role: &Role) -> impl Future<Output = Result<Role, CoreError>> + Send;

    fn delete(&mut self, id: RoleId) -> impl Future<Output = Result<(), CoreError>> + Send;

    /// How many members currently hold this role — `delete` refuses above
    /// zero (#308): `member_roles.role_id` cascades on delete, so deleting a
    /// role still assigned would silently strip those members' permissions
    /// rather than the org choosing to.
    fn count_assigned_members(
        &mut self,
        id: RoleId,
    ) -> impl Future<Output = Result<i64, CoreError>> + Send;
}
