use std::str::FromStr;

use chrono::{DateTime, Utc};
use common::CoreError;
use rust_decimal::Decimal;
use uuid::Uuid;

use crate::{
	CustomerContextId, CustomerId, OrganizationContextId, OrganizationId, ServiceRateId,
	ServiceRateUnit,
	domain::{
		invoice::{
			Invoice, InvoiceId, InvoiceLine, InvoiceLineId, InvoiceStatus, InvoiceType,
		},
		quote::QuoteId,
	},
};

#[derive(Debug, Clone)]
pub struct InvoiceRow {
	pub id: Uuid,
	pub org_id: Uuid,
	pub customer_id: Uuid,
	pub customer_context_id: Uuid,
	pub emitter_context_id: Option<Uuid>,
	pub reference: Option<String>,
	pub title: String,
	pub status: String,
	pub invoice_type: String,
	pub source_quote_id: Option<Uuid>,
	pub parent_invoice_id: Option<Uuid>,
	pub deposit_basis: Option<String>,
	pub deposit_value: Option<Decimal>,
	pub deposit_amount_cents: Option<i32>,
	pub total_ht_cents: i32,
	pub total_vat_cents: i32,
	pub total_ttc_cents: i32,
	pub total_cents: i32,
	pub amount_paid_cents: i32,
	pub pdf_key: Option<String>,
	pub issued_at: Option<DateTime<Utc>>,
	pub due_at: Option<DateTime<Utc>>,
	pub deleted_at: Option<DateTime<Utc>>,
	pub created_at: DateTime<Utc>,
	pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct InvoiceLineRow {
	pub id: Uuid,
	pub org_id: Uuid,
	pub invoice_id: Uuid,
	pub service_rate_id: Option<Uuid>,
	pub label: String,
	pub quantity: Decimal,
	pub unit: String,
	pub unit_price_cents: i32,
	pub vat_rate: Decimal,
	pub notes: Option<String>,
	pub photo_keys: Vec<String>,
	pub deleted_at: Option<DateTime<Utc>>,
	pub created_at: DateTime<Utc>,
	pub updated_at: DateTime<Utc>,
}

impl InvoiceRow {
	pub fn into_invoice(self, lines: Vec<InvoiceLine>) -> Result<Invoice, CoreError> {
		let status = InvoiceStatus::from_str(&self.status).map_err(|e| {
			CoreError::Internal(format!("invalid invoice status in database: {e}"))
		})?;
		let invoice_type = InvoiceType::from_str(&self.invoice_type).map_err(|e| {
			CoreError::Internal(format!("invalid invoice type in database: {e}"))
		})?;

		Ok(Invoice {
			id: InvoiceId(self.id),
			org_id: OrganizationId(self.org_id),
			customer_id: CustomerId(self.customer_id),
			customer_context_id: CustomerContextId(self.customer_context_id),
			emitter_context_id: self.emitter_context_id.map(OrganizationContextId),
			reference: self.reference,
			title: self.title,
			status,
			invoice_type,
			source_quote_id: self.source_quote_id.map(QuoteId),
			parent_invoice_id: self.parent_invoice_id.map(InvoiceId),
			deposit_basis: self.deposit_basis,
			deposit_value: self.deposit_value,
			deposit_amount_cents: self.deposit_amount_cents,
			total_ht_cents: self.total_ht_cents,
			total_vat_cents: self.total_vat_cents,
			total_ttc_cents: self.total_ttc_cents,
			total_cents: self.total_cents,
			amount_paid_cents: self.amount_paid_cents,
			pdf_key: self.pdf_key,
			issued_at: self.issued_at,
			due_at: self.due_at,
			lines,
			// Populated by service layer after a separate fetch.
			legal_mention_template_ids: vec![],
			deleted_at: self.deleted_at,
			created_at: self.created_at,
			updated_at: self.updated_at,
		})
	}
}

impl TryFrom<InvoiceLineRow> for InvoiceLine {
	type Error = CoreError;

	fn try_from(row: InvoiceLineRow) -> Result<Self, Self::Error> {
		let unit = ServiceRateUnit::from_str(&row.unit).map_err(|e| {
			CoreError::Internal(format!("invalid invoice line unit in database: {e}"))
		})?;

		Ok(Self {
			id: InvoiceLineId(row.id),
			org_id: OrganizationId(row.org_id),
			invoice_id: InvoiceId(row.invoice_id),
			service_rate_id: row.service_rate_id.map(ServiceRateId),
			label: row.label,
			quantity: row.quantity,
			unit,
			unit_price_cents: row.unit_price_cents,
			vat_rate: row.vat_rate,
			notes: row.notes,
			photo_keys: row.photo_keys,
			deleted_at: row.deleted_at,
			created_at: row.created_at,
			updated_at: row.updated_at,
		})
	}
}
