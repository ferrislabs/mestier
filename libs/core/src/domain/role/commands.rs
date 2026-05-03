use crate::domain::{organization::OrganizationId, role::Permissions};

#[derive(Debug, Clone)]
pub struct CreateRoleCommand {
    pub organization_id: OrganizationId,
    pub name: String,
    pub permissions: Permissions,
}
