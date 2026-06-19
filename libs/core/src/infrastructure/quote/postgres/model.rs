use std::str::FromStr;

use chrono::{DateTime, Utc};
use common::CoreError;
use rust_decimal::Decimal;
use uuid::Uuid;

use crate::{
    CustomerContextId, CustomerId, OrganizationId, Quote, QuoteId, QuoteLine, QuoteLineId,
    QuoteStatus, ServiceRateId, ServiceRateUnit,
};

#[derive(Debug, Clone)]
pub struct QuoteRow {
    pub id: Uuid,
    pub org_id: Uuid,
    pub customer_id: Uuid,
    pub customer_context_id: Uuid,
    pub status: String,
    pub total_cents: i32,
    pub deleted_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct QuoteLineRow {
    pub id: Uuid,
    pub org_id: Uuid,
    pub quote_id: Uuid,
    pub service_rate_id: Option<Uuid>,
    pub label: String,
    pub quantity: Decimal,
    pub unit: String,
    pub unit_price_cents: i32,
    pub notes: Option<String>,
    pub photo_keys: Vec<String>,
    pub deleted_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl QuoteRow {
    pub fn into_quote(self, lines: Vec<QuoteLine>) -> Result<Quote, CoreError> {
        let status = QuoteStatus::from_str(&self.status)
            .map_err(|e| CoreError::Internal(format!("invalid quote status in database: {e}")))?;

        Ok(Quote {
            id: QuoteId(self.id),
            organization_id: OrganizationId(self.org_id),
            customer_id: CustomerId(self.customer_id),
            customer_context_id: CustomerContextId(self.customer_context_id),
            status,
            total_cents: self.total_cents,
            lines,
            deleted_at: self.deleted_at,
            created_at: self.created_at,
            updated_at: self.updated_at,
        })
    }
}

impl TryFrom<QuoteLineRow> for QuoteLine {
    type Error = CoreError;

    fn try_from(row: QuoteLineRow) -> Result<Self, Self::Error> {
        let unit = ServiceRateUnit::from_str(&row.unit).map_err(|e| {
            CoreError::Internal(format!("invalid quote line unit in database: {e}"))
        })?;

        Ok(Self {
            id: QuoteLineId(row.id),
            organization_id: OrganizationId(row.org_id),
            quote_id: QuoteId(row.quote_id),
            service_rate_id: row.service_rate_id.map(ServiceRateId),
            label: row.label,
            quantity: row.quantity,
            unit,
            unit_price_cents: row.unit_price_cents,
            notes: row.notes,
            photo_keys: row.photo_keys,
            deleted_at: row.deleted_at,
            created_at: row.created_at,
            updated_at: row.updated_at,
        })
    }
}
