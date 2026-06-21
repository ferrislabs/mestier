use chrono::{DateTime, Utc};
use mestier_core::{
    CustomerContextId, CustomerId, OrganizationId, Quote, QuoteId, QuoteLine, QuoteLineId,
    QuoteStatus, ServiceRateId, ServiceRateUnit,
};
use serde::Serialize;
use utoipa::ToSchema;

#[derive(Debug, Clone, PartialEq, Serialize, ToSchema)]
pub struct QuoteLineResponse {
    pub id: QuoteLineId,
    pub organization_id: OrganizationId,
    pub quote_id: QuoteId,
    pub service_rate_id: Option<ServiceRateId>,
    pub label: String,
    pub quantity: String,
    pub unit: ServiceRateUnit,
    pub unit_price_cents: i32,
    pub notes: Option<String>,
    pub photo_keys: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<QuoteLine> for QuoteLineResponse {
    fn from(value: QuoteLine) -> Self {
        Self {
            id: value.id,
            organization_id: value.organization_id,
            quote_id: value.quote_id,
            service_rate_id: value.service_rate_id,
            label: value.label,
            quantity: value.quantity.normalize().to_string(),
            unit: value.unit,
            unit_price_cents: value.unit_price_cents,
            notes: value.notes,
            photo_keys: value.photo_keys,
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, ToSchema)]
pub struct QuoteResponse {
    pub id: QuoteId,
    pub organization_id: OrganizationId,
    pub reference: String,
    pub title: String,
    pub customer_id: CustomerId,
    pub customer_context_id: CustomerContextId,
    pub status: QuoteStatus,
    pub total_cents: i32,
    pub lines: Vec<QuoteLineResponse>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<Quote> for QuoteResponse {
    fn from(value: Quote) -> Self {
        Self {
            id: value.id,
            organization_id: value.organization_id,
            reference: value.reference,
            title: value.title,
            customer_id: value.customer_id,
            customer_context_id: value.customer_context_id,
            status: value.status,
            total_cents: value.total_cents,
            lines: value
                .lines
                .into_iter()
                .map(QuoteLineResponse::from)
                .collect(),
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}
