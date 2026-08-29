use crate::{EquipmentId, OrganizationId};

#[derive(Debug, Clone)]
pub struct CreateEquipmentCommand {
    /// Authenticated actor performing the update. Built by the handler from
    /// the request `Identity`; carries the AuthZen-shaped subject the policy
    /// engine consumes.
    pub actor: authz::Subject,
    pub organization_id: OrganizationId,
    pub name: String,
    pub hourly_rate_cents: i32,
}

#[derive(Debug, Clone)]
pub struct UpdateEquipmentCommand {
    /// Authenticated actor performing the update. Built by the handler from
    /// the request `Identity`; carries the AuthZen-shaped subject the policy
    /// engine consumes.
    pub actor: authz::Subject,
    pub id: EquipmentId,
    pub name: String,
    pub hourly_rate_cents: i32,
}
