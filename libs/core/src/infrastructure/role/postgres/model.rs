use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::domain::{
    organization::OrganizationId,
    role::{Permissions, Role, RoleId},
};

#[derive(Debug, Clone)]
pub struct RoleRow {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub name: String,
    pub permissions: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<RoleRow> for Role {
    fn from(row: RoleRow) -> Self {
        Self {
            id: RoleId(row.id),
            organization_id: OrganizationId(row.organization_id),
            name: row.name,
            permissions: Permissions(row.permissions),
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}
