use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::{CustomerContact, CustomerContactId, CustomerId};

#[derive(Debug, Clone)]
pub struct CustomerContactRow {
    pub id: Uuid,
    pub customer_id: Uuid,
    pub first_name: String,
    pub last_name: String,
    pub role: Option<String>,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub is_primary: bool,
    pub deleted_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<CustomerContactRow> for CustomerContact {
    fn from(row: CustomerContactRow) -> Self {
        Self {
            id: CustomerContactId(row.id),
            customer_id: CustomerId(row.customer_id),
            first_name: row.first_name,
            last_name: row.last_name,
            role: row.role,
            phone: row.phone,
            email: row.email,
            is_primary: row.is_primary,
            deleted_at: row.deleted_at,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}
