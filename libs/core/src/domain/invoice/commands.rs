use chrono::{DateTime, Utc};
use rust_decimal::Decimal;

use crate::{
	CustomerContextId, CustomerId, LegalMentionTemplateId, OrganizationId, ServiceRateId,
	ServiceRateUnit,
};
use crate::domain::invoice::{InvoiceId, InvoiceStatus, InvoiceType};
use crate::domain::quote::QuoteId;

#[derive(Debug, Clone)]
pub struct InvoiceLineCommand {
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
pub struct CreateInvoiceCommand {
	pub org_id: OrganizationId,
	pub title: String,
	pub customer_id: CustomerId,
	pub customer_context_id: CustomerContextId,
	pub invoice_type: InvoiceType,
	pub source_quote_id: Option<QuoteId>,
	pub parent_invoice_id: Option<InvoiceId>,
	pub deposit_basis: Option<String>,
	pub deposit_value: Option<Decimal>,
	pub deposit_amount_cents: Option<i32>,
	pub due_at: Option<DateTime<Utc>>,
	pub lines: Vec<InvoiceLineCommand>,
	pub legal_mention_template_ids: Vec<LegalMentionTemplateId>,
}

#[derive(Debug, Clone)]
pub struct UpdateInvoiceCommand {
	pub id: InvoiceId,
	pub title: String,
	pub customer_id: CustomerId,
	pub customer_context_id: CustomerContextId,
	pub deposit_basis: Option<String>,
	pub deposit_value: Option<Decimal>,
	pub deposit_amount_cents: Option<i32>,
	pub due_at: Option<DateTime<Utc>>,
	pub lines: Vec<InvoiceLineCommand>,
	pub legal_mention_template_ids: Vec<LegalMentionTemplateId>,
}

#[derive(Debug, Clone, Copy)]
pub struct UpdateInvoiceStatusCommand {
	pub id: InvoiceId,
	pub status: InvoiceStatus,
}
