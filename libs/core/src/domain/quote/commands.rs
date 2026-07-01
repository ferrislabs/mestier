use rust_decimal::Decimal;

use crate::{
	CustomerContextId, CustomerId, LegalMentionTemplateId, OrganizationId, QuoteId, QuoteStatus,
	ServiceRateId, ServiceRateUnit,
};

#[derive(Debug, Clone)]
pub struct QuoteLineCommand {
    pub service_rate_id: Option<ServiceRateId>,
    pub label: String,
    pub quantity: Decimal,
    pub unit: ServiceRateUnit,
    pub unit_price_cents: i32,
    pub vat_rate: Decimal,
    pub notes: Option<String>,
    pub photo_keys: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct CreateQuoteCommand {
	pub organization_id: OrganizationId,
	pub title: String,
	pub customer_id: CustomerId,
	pub customer_context_id: CustomerContextId,
	pub deposit_basis: Option<String>,
	pub deposit_value: Option<Decimal>,
	pub lines: Vec<QuoteLineCommand>,
	pub legal_mention_template_ids: Vec<LegalMentionTemplateId>,
}

#[derive(Debug, Clone)]
pub struct UpdateQuoteCommand {
	pub id: QuoteId,
	pub title: String,
	pub customer_id: CustomerId,
	pub customer_context_id: CustomerContextId,
	pub status: QuoteStatus,
	pub deposit_basis: Option<String>,
	pub deposit_value: Option<Decimal>,
	pub lines: Vec<QuoteLineCommand>,
	pub legal_mention_template_ids: Vec<LegalMentionTemplateId>,
}

#[derive(Debug, Clone, Copy)]
pub struct UpdateQuoteStatusCommand {
    pub id: QuoteId,
    pub status: QuoteStatus,
}
