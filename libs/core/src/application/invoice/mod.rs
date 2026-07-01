use common::CoreError;
use mestier_macros::transactional;

use crate::{
	Invoice, InvoiceId, OrganizationId,
	application::MestierUseCase,
	domain::invoice::{
		InvoiceType,
		commands::{CreateInvoiceCommand, InvoiceLineCommand, UpdateInvoiceCommand, UpdateInvoiceStatusCommand},
		service::InvoiceService,
	},
	domain::quote::{QuoteId, service::QuoteService},
};

impl MestierUseCase {
	#[transactional(invoice)]
	pub async fn create_invoice(
		&self,
		command: CreateInvoiceCommand,
	) -> Result<Invoice, CoreError> {
		let mut service = InvoiceService::new(invoice_repository);
		service.create_invoice(command).await
	}

	#[transactional(invoice)]
	pub async fn get_invoice(&self, id: InvoiceId) -> Result<Invoice, CoreError> {
		let mut service = InvoiceService::new(invoice_repository);
		service.get_invoice(id).await
	}

	#[transactional(invoice)]
	pub async fn list_invoices(
		&self,
		org_id: OrganizationId,
		limit: u64,
		offset: u64,
	) -> Result<(Vec<Invoice>, u64), CoreError> {
		let mut service = InvoiceService::new(invoice_repository);
		service.list_invoices(org_id, limit, offset).await
	}

	#[transactional(invoice)]
	pub async fn update_invoice(
		&self,
		command: UpdateInvoiceCommand,
	) -> Result<Invoice, CoreError> {
		let mut service = InvoiceService::new(invoice_repository);
		service.update_invoice(command).await
	}

	#[transactional(invoice)]
	pub async fn update_invoice_status(
		&self,
		command: UpdateInvoiceStatusCommand,
	) -> Result<Invoice, CoreError> {
		let mut service = InvoiceService::new(invoice_repository);
		service.update_invoice_status(command).await
	}

	#[transactional(invoice)]
	pub async fn soft_delete_invoice(&self, id: InvoiceId) -> Result<(), CoreError> {
		let mut service = InvoiceService::new(invoice_repository);
		service.soft_delete_invoice(id).await
	}

	#[transactional(quote, invoice)]
	pub async fn convert_quote_to_invoice(
		&self,
		quote_id: QuoteId,
	) -> Result<Invoice, CoreError> {
		let mut quote_service = QuoteService::new(quote_repository);
		let quote = quote_service.get_quote(quote_id).await?;

		let lines = quote
			.lines
			.iter()
			.filter(|l| l.deleted_at.is_none())
			.map(|l| InvoiceLineCommand {
				service_rate_id: l.service_rate_id,
				label: l.label.clone(),
				quantity: l.quantity,
				unit: l.unit,
				unit_price_cents: l.unit_price_cents,
				vat_rate: l.vat_rate,
				notes: l.notes.clone(),
				photo_keys: l.photo_keys.clone(),
			})
			.collect();

		let command = CreateInvoiceCommand {
			org_id: quote.organization_id,
			title: quote.title.clone(),
			customer_id: quote.customer_id,
			customer_context_id: quote.customer_context_id,
			invoice_type: InvoiceType::Standard,
			source_quote_id: Some(quote.id),
			parent_invoice_id: None,
			deposit_basis: None,
			deposit_value: None,
			deposit_amount_cents: None,
			due_at: None,
			lines,
			legal_mention_template_ids: quote.legal_mention_template_ids.clone(),
		};

		let mut invoice_service = InvoiceService::new(invoice_repository);
		invoice_service.create_invoice(command).await
	}
}
