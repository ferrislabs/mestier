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
    pub org_id: OrganizationId,
    pub quote_id: QuoteId,
    pub service_rate_id: Option<ServiceRateId>,
    pub label: String,
    pub quantity: String,
    pub unit: ServiceRateUnit,
    pub unit_price_cents: i32,
    pub vat_rate: String,
    pub notes: Option<String>,
    pub photo_keys: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<QuoteLine> for QuoteLineResponse {
    fn from(value: QuoteLine) -> Self {
        Self {
            id: value.id,
            org_id: value.organization_id,
            quote_id: value.quote_id,
            service_rate_id: value.service_rate_id,
            label: value.label,
            quantity: value.quantity.normalize().to_string(),
            unit: value.unit,
            unit_price_cents: value.unit_price_cents,
            vat_rate: value.vat_rate.normalize().to_string(),
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
    pub org_id: OrganizationId,
    pub reference: String,
    pub title: String,
    pub customer_id: CustomerId,
    pub customer_context_id: CustomerContextId,
    pub emitter_context_id: Option<String>,
    pub status: QuoteStatus,
    pub deposit_basis: Option<String>,
    pub deposit_value: Option<String>,
    pub total_cents: i32,
    pub total_ht_cents: i32,
    pub total_vat_cents: i32,
    pub total_ttc_cents: i32,
    pub lines: Vec<QuoteLineResponse>,
    pub legal_mention_template_ids: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<Quote> for QuoteResponse {
    fn from(value: Quote) -> Self {
        Self {
            id: value.id,
            org_id: value.organization_id,
            reference: value.reference,
            title: value.title,
            customer_id: value.customer_id,
            customer_context_id: value.customer_context_id,
            emitter_context_id: value.emitter_context_id.map(|id| id.0.to_string()),
            status: value.status,
            deposit_basis: value.deposit_basis,
            deposit_value: value.deposit_value.map(|d| d.normalize().to_string()),
            total_cents: value.total_cents,
            total_ht_cents: value.total_ht_cents,
            total_vat_cents: value.total_vat_cents,
            total_ttc_cents: value.total_ttc_cents,
            lines: value
                .lines
                .into_iter()
                .map(QuoteLineResponse::from)
                .collect(),
            legal_mention_template_ids: value
                .legal_mention_template_ids
                .into_iter()
                .map(|id| id.0.to_string())
                .collect(),
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}
