use chrono::{DateTime, Utc};
use discord::{Category, CategoryId, OrganizationId};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct CategoryRow {
    pub id: Uuid,
    pub org_id: Uuid,
    pub name: String,
    pub position: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<CategoryRow> for Category {
    fn from(r: CategoryRow) -> Self {
        Self {
            id: CategoryId(r.id),
            organization_id: OrganizationId(r.org_id),
            name: r.name,
            position: r.position,
            created_at: r.created_at,
            updated_at: r.updated_at,
        }
    }
}
