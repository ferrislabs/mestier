use chrono::{DateTime, Utc};
use uuid::Uuid;

use common::CoreError;
use std::str::FromStr;

use crate::{AddressKind, CustomerContext, CustomerContextId, CustomerId};

#[derive(Debug, Clone)]
pub struct CustomerContextRow {
    pub id: Uuid,
    pub customer_id: Uuid,
    pub label: String,
    pub kind: String,
    pub address_line: Option<String>,
    pub address_line2: Option<String>,
    pub postal_code: Option<String>,
    pub city: Option<String>,
    pub country: String,
    pub photo_key: Option<String>,
    pub deleted_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl TryFrom<CustomerContextRow> for CustomerContext {
    type Error = CoreError;

    fn try_from(row: CustomerContextRow) -> Result<Self, Self::Error> {
        let kind = AddressKind::from_str(&row.kind)
            .map_err(|e| CoreError::Internal(format!("invalid address kind in database: {e}")))?;

        Ok(Self {
            id: CustomerContextId(row.id),
            customer_id: CustomerId(row.customer_id),
            label: row.label,
            kind,
            address_line: row.address_line,
            address_line2: row.address_line2,
            postal_code: row.postal_code,
            city: row.city,
            country: row.country,
            photo_key: row.photo_key,
            deleted_at: row.deleted_at,
            created_at: row.created_at,
            updated_at: row.updated_at,
        })
    }
}
