use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::{CustomerContext, CustomerContextId, CustomerId};

#[derive(Debug, Clone)]
pub struct CustomerContextRow {
    pub id: Uuid,
    pub customer_id: Uuid,
    pub label: String,
    pub address_line: Option<String>,
    pub postal_code: Option<String>,
    pub city: Option<String>,
    pub photo_key: Option<String>,
    pub deleted_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<CustomerContextRow> for CustomerContext {
    fn from(row: CustomerContextRow) -> Self {
        Self {
            id: CustomerContextId(row.id),
            customer_id: CustomerId(row.customer_id),
            label: row.label,
            address_line: row.address_line,
            postal_code: row.postal_code,
            city: row.city,
            photo_key: row.photo_key,
            deleted_at: row.deleted_at,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}
