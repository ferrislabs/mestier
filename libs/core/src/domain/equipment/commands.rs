use crate::{EquipmentId, OrganizationId};

#[derive(Debug, Clone)]
pub struct CreateEquipmentCommand {
    pub organization_id: OrganizationId,
    pub name: String,
    pub hourly_rate_cents: i32,
}

#[derive(Debug, Clone)]
pub struct UpdateEquipmentCommand {
    pub id: EquipmentId,
    pub name: String,
    pub hourly_rate_cents: i32,
}
