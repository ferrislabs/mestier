use authz::Subject;

use crate::{UserId, domain::organization::OrganizationId};

#[derive(Debug, Clone)]
pub struct CreateOrganizationCommand {
    pub name: String,
    pub slug: String,
    pub owner_id: UserId,
}

#[derive(Debug, Clone)]
pub struct UpdateOrganizationCommand {
    /// Authenticated actor performing the update. Built by the handler
    /// from the request `Identity`; carries the AuthZen-shaped subject
    /// the policy engine consumes.
    pub actor: Subject,
    pub id: OrganizationId,
    pub name: String,
    pub slug: String,
}
