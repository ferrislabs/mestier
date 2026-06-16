use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::{Equipment, EquipmentId, OrganizationId};

#[derive(Debug, Clone)]
pub struct EquipmentRow {
    pub id: Uuid,
    pub org_id: Uuid,
    pub name: String,
    pub hourly_rate_cents: i32,
    pub deleted_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<EquipmentRow> for Equipment {
    fn from(row: EquipmentRow) -> Self {
        Self {
            id: EquipmentId(row.id),
            organization_id: OrganizationId(row.org_id),
            name: row.name,
            hourly_rate_cents: row.hourly_rate_cents,
            deleted_at: row.deleted_at,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}
