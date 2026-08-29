use authz::Subject;

use crate::domain::{
    organization::OrganizationId,
    role::{Permissions, RoleId},
};

#[derive(Debug, Clone)]
pub struct CreateRoleCommand {
    /// Authenticated actor performing the update. Built by the handler
    /// from the request `Identity`; carries the AuthZen-shaped subject
    /// the policy engine consumes.
    pub actor: Subject,
    pub organization_id: OrganizationId,
    pub name: String,
    pub permissions: Permissions,
}

#[derive(Debug, Clone)]
pub struct UpdateRoleCommand {
    /// Authenticated actor performing the update. Built by the handler
    /// from the request `Identity`; carries the AuthZen-shaped subject
    /// the policy engine consumes.
    pub actor: Subject,
    pub role_id: RoleId,
    pub name: String,
    pub permissions: Permissions,
}
