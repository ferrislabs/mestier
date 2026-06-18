use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::{Customer, CustomerId, OrganizationId};

#[derive(Debug, Clone)]
pub struct CustomerRow {
    pub id: Uuid,
    pub org_id: Uuid,
    pub last_name: String,
    pub first_name: String,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub deleted_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<CustomerRow> for Customer {
    fn from(row: CustomerRow) -> Self {
        Self {
            id: CustomerId(row.id),
            organization_id: OrganizationId(row.org_id),
            last_name: row.last_name,
            first_name: row.first_name,
            phone: row.phone,
            email: row.email,
            deleted_at: row.deleted_at,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}
