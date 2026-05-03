use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::{
    UserId,
    domain::organization::{Organization, OrganizationId},
};

#[derive(Debug, Clone)]
pub struct OrganizationRow {
    pub id: Uuid,
    pub name: String,
    pub slug: String,
    pub owner_id: Uuid,
    pub deleted_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<OrganizationRow> for Organization {
    fn from(row: OrganizationRow) -> Self {
        Self {
            id: OrganizationId(row.id),
            name: row.name,
            slug: row.slug,
            owner_id: UserId(row.owner_id),
            deleted_at: row.deleted_at,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}
