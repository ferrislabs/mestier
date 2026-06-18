use rust_decimal::Decimal;

use crate::{
    CustomerId, OrganizationId, PropertyId, QuoteId, QuoteStatus, ServiceRateId, ServiceRateUnit,
};

#[derive(Debug, Clone)]
pub struct QuoteLineCommand {
    pub service_rate_id: Option<ServiceRateId>,
    pub label: String,
    pub quantity: Decimal,
    pub unit: ServiceRateUnit,
    pub unit_price_cents: i32,
    pub notes: Option<String>,
    pub photo_keys: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct CreateQuoteCommand {
    pub organization_id: OrganizationId,
    pub customer_id: CustomerId,
    pub property_id: PropertyId,
    pub lines: Vec<QuoteLineCommand>,
}

#[derive(Debug, Clone)]
pub struct UpdateQuoteCommand {
    pub id: QuoteId,
    pub customer_id: CustomerId,
    pub property_id: PropertyId,
    pub status: QuoteStatus,
    pub lines: Vec<QuoteLineCommand>,
}

#[derive(Debug, Clone, Copy)]
pub struct UpdateQuoteStatusCommand {
    pub id: QuoteId,
    pub status: QuoteStatus,
}
