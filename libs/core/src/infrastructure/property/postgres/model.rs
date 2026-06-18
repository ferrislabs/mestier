use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::{CustomerId, Property, PropertyId};

#[derive(Debug, Clone)]
pub struct PropertyRow {
    pub id: Uuid,
    pub customer_id: Uuid,
    pub label: String,
    pub street: String,
    pub zip: String,
    pub city: String,
    pub photo_key: Option<String>,
    pub deleted_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<PropertyRow> for Property {
    fn from(row: PropertyRow) -> Self {
        Self {
            id: PropertyId(row.id),
            customer_id: CustomerId(row.customer_id),
            label: row.label,
            street: row.street,
            zip: row.zip,
            city: row.city,
            photo_key: row.photo_key,
            deleted_at: row.deleted_at,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}
