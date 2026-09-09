use crate::OrganizationId;

#[derive(Debug, Clone)]
pub struct CreateCustomUnitCommand {
    /// Authenticated actor performing the update. Built by the handler from
    /// the request `Identity`; carries the AuthZen-shaped subject the policy
    /// engine consumes.
    pub actor: authz::Subject,
    pub organization_id: OrganizationId,
    pub code: String,
}
