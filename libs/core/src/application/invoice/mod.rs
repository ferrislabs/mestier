use common::CoreError;
use mestier_macros::transactional;

use crate::{
	Invoice, InvoiceId, OrganizationId,
	application::MestierUseCase,
	domain::invoice::{
		commands::{CreateInvoiceCommand, UpdateInvoiceCommand, UpdateInvoiceStatusCommand},
		service::InvoiceService,
	},
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
}
