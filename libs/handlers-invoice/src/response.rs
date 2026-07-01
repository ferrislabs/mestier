use chrono::{DateTime, Utc};
use mestier_core::{
	CustomerContextId, CustomerId, Invoice, InvoiceId, InvoiceLine, InvoiceLineId, InvoiceStatus,
	InvoiceType, OrganizationId, ServiceRateId, ServiceRateUnit,
};
use serde::Serialize;
use utoipa::ToSchema;

#[derive(Debug, Clone, PartialEq, Serialize, ToSchema)]
pub struct InvoiceLineResponse {
	pub id: InvoiceLineId,
	pub org_id: OrganizationId,
	pub invoice_id: InvoiceId,
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

impl From<InvoiceLine> for InvoiceLineResponse {
	fn from(value: InvoiceLine) -> Self {
		Self {
			id: value.id,
			org_id: value.org_id,
			invoice_id: value.invoice_id,
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
pub struct InvoiceResponse {
	pub id: InvoiceId,
	pub org_id: OrganizationId,
	pub customer_id: CustomerId,
	pub customer_context_id: CustomerContextId,
	pub emitter_context_id: Option<String>,
	pub reference: Option<String>,
	pub title: String,
	pub status: InvoiceStatus,
	pub invoice_type: InvoiceType,
	pub source_quote_id: Option<String>,
	pub parent_invoice_id: Option<String>,
	pub deposit_basis: Option<String>,
	pub deposit_value: Option<String>,
	pub deposit_amount_cents: Option<i32>,
	pub total_ht_cents: i32,
	pub total_vat_cents: i32,
	pub total_ttc_cents: i32,
	pub total_cents: i32,
	pub amount_paid_cents: i32,
	pub pdf_key: Option<String>,
	pub issued_at: Option<DateTime<Utc>>,
	pub due_at: Option<DateTime<Utc>>,
	pub lines: Vec<InvoiceLineResponse>,
	pub legal_mention_template_ids: Vec<String>,
	pub created_at: DateTime<Utc>,
	pub updated_at: DateTime<Utc>,
}

impl From<Invoice> for InvoiceResponse {
	fn from(value: Invoice) -> Self {
		Self {
			id: value.id,
			org_id: value.org_id,
			customer_id: value.customer_id,
			customer_context_id: value.customer_context_id,
			emitter_context_id: value.emitter_context_id.map(|id| id.0.to_string()),
			reference: value.reference,
			title: value.title,
			status: value.status,
			invoice_type: value.invoice_type,
			source_quote_id: value.source_quote_id.map(|id| id.0.to_string()),
			parent_invoice_id: value.parent_invoice_id.map(|id| id.0.to_string()),
			deposit_basis: value.deposit_basis,
			deposit_value: value.deposit_value.map(|d| d.normalize().to_string()),
			deposit_amount_cents: value.deposit_amount_cents,
			total_ht_cents: value.total_ht_cents,
			total_vat_cents: value.total_vat_cents,
			total_ttc_cents: value.total_ttc_cents,
			total_cents: value.total_cents,
			amount_paid_cents: value.amount_paid_cents,
			pdf_key: value.pdf_key,
			issued_at: value.issued_at,
			due_at: value.due_at,
			lines: value
				.lines
				.into_iter()
				.map(InvoiceLineResponse::from)
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
