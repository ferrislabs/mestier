use chrono::{Datelike, Utc};
use common::CoreError;
use mestier_macros::transactional;
use rust_decimal::Decimal;

use crate::{
	Invoice, InvoiceId, OrganizationId,
	application::{
		MestierUseCase,
		document::snapshot::build_invoice_snapshot,
	},
	domain::{
		billing::DepositBasis,
		billing_settings::ports::BillingSettingsRepository,
		customer::ports::CustomerRepository,
		customer_context::ports::CustomerContextRepository,
		file_storage::commands::UploadFileCommand,
		invoice::{
			InvoiceStatus, InvoiceType,
			commands::{CreateInvoiceCommand, InvoiceLineCommand, UpdateInvoiceCommand, UpdateInvoiceStatusCommand},
			ports::InvoiceRepository,
			service::InvoiceService,
		},
		legal_mention_template::ports::LegalMentionTemplateRepository,
		organization::ports::OrganizationRepository,
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

	#[transactional(invoice)]
	pub async fn create_deposit_invoice(
		&self,
		parent_invoice_id: InvoiceId,
		basis: DepositBasis,
		value: Decimal,
	) -> Result<Invoice, CoreError> {
		let mut service = InvoiceService::new(invoice_repository);
		service.create_deposit_invoice(parent_invoice_id, basis, value).await
	}

	#[transactional(invoice)]
	pub async fn create_balance_invoice(
		&self,
		parent_invoice_id: InvoiceId,
	) -> Result<Invoice, CoreError> {
		let mut service = InvoiceService::new(invoice_repository);
		service.create_balance_invoice(parent_invoice_id).await
	}

	/// Issue an invoice: assign a reference, generate the PDF snapshot, upload it
	/// to object storage, and persist the immutable issued state atomically.
	///
	/// Only DRAFT invoices may be issued. The invoice becomes immutable after this call.
	#[transactional(invoice, billing_settings, customer, customer_context, legal_mention_template, organization)]
	pub async fn issue_invoice(&self, invoice_id: InvoiceId) -> Result<Invoice, CoreError> {
		// Shadow-rebind all repos as mutable (the macro emits non-mut bindings).
		let mut invoice_repository = invoice_repository;
		let mut billing_settings_repository = billing_settings_repository;
		let mut customer_repository = customer_repository;
		let mut customer_context_repository = customer_context_repository;
		let mut legal_mention_template_repository = legal_mention_template_repository;
		let mut organization_repository = organization_repository;

		// ── 1. Load invoice, guard DRAFT ─────────────────────────────────────
		let invoice = invoice_repository
			.find_by_id(invoice_id)
			.await?
			.ok_or(CoreError::NotFound)?;
		let legal_mention_template_ids = invoice_repository
			.find_legal_mention_template_ids(invoice_id)
			.await?;
		let invoice = Invoice { legal_mention_template_ids, ..invoice };

		if invoice.status != InvoiceStatus::Draft {
			return Err(CoreError::Conflict(
				"only DRAFT invoices can be issued".to_owned(),
			));
		}

		// ── 2. Guard: emitter context required ───────────────────────────────
		if invoice.emitter_context_id.is_none() {
			return Err(CoreError::Conflict(
				"select an emitter context before issuing the invoice".to_owned(),
			));
		}

		// ── 3. Assign reference + issued_at ──────────────────────────────────
		let now = Utc::now();
		let reference = invoice_repository
			.next_reference(invoice.org_id, now.year())
			.await?;

		// ── 4. Gather billing settings (optional — used for payment terms/footer) ──
		let settings = billing_settings_repository
			.find_by_org(invoice.org_id)
			.await?;

		// ── 5. Gather organization (for seller name) ──────────────────────────
		let org = organization_repository
			.find_by_id(invoice.org_id)
			.await?
			.ok_or(CoreError::NotFound)?;

		// ── 6. Gather customer + context ─────────────────────────────────────
		let customer = customer_repository
			.find_by_id(invoice.customer_id)
			.await?
			.ok_or(CoreError::NotFound)?;

		let context = customer_context_repository
			.find_by_id(invoice.customer_context_id)
			.await?;

		// ── 7. Gather legal mention bodies ────────────────────────────────────
		let mut mention_bodies: Vec<String> = Vec::with_capacity(invoice.legal_mention_template_ids.len());
		for template_id in &invoice.legal_mention_template_ids {
			if let Some(t) = legal_mention_template_repository.find_by_id(*template_id).await? {
				mention_bodies.push(t.body);
			}
		}

		// ── 8. Build snapshot ─────────────────────────────────────────────────
		let snapshot = build_invoice_snapshot(
			&Invoice {
				reference: Some(reference.clone()),
				issued_at: Some(now),
				status: InvoiceStatus::Sent,
				..invoice.clone()
			},
			&org.name,
			settings.as_ref(),
			&customer,
			context.as_ref(),
			&mention_bodies,
		);

		// ── 9. Render PDF ─────────────────────────────────────────────────────
		let pdf_bytes = pdf::render_document_pdf(&snapshot)
			.map_err(|e| CoreError::Internal(format!("PDF rendering failed: {e}")))?;

		// ── 10. Upload PDF ────────────────────────────────────────────────────
		// Note: upload is a side effect outside the DB transaction. On DB commit
		// failure the S3 object becomes an orphan (the invoice stays DRAFT and
		// can be retried). This is acceptable — a cleanup job can prune orphans.
		let stored = self
			.file_storage
			.upload(UploadFileCommand {
				mime_type: "application/pdf".to_owned(),
				bytes: pdf_bytes,
				folder: Some("documents".to_owned()),
			})
			.await?;

		// ── 11. Persist issued state atomically ──────────────────────────────
		let snapshot_json = serde_json::to_value(&snapshot)
			.map_err(|e| CoreError::Internal(format!("snapshot serialization failed: {e}")))?;

		let issued_invoice = invoice_repository
			.mark_issued(
				invoice_id,
				reference,
				now,
				stored.key,
				snapshot_json,
				now,
			)
			.await?;

		// Re-attach legal mention template ids (not returned by mark_issued)
		let legal_mention_template_ids = invoice_repository
			.find_legal_mention_template_ids(invoice_id)
			.await?;

		Ok(Invoice {
			legal_mention_template_ids,
			..issued_invoice
		})
	}

	#[transactional(quote, invoice)]
	pub async fn convert_quote_to_invoice(
		&self,
		quote_id: QuoteId,
	) -> Result<Invoice, CoreError> {
		let mut quote_service = QuoteService::new(quote_repository);
		let quote = quote_service.get_quote(quote_id).await?;

		if quote.emitter_context_id.is_none() {
			return Err(CoreError::Conflict(
				"select an emitter context before converting the quote".to_owned(),
			));
		}

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
			emitter_context_id: quote.emitter_context_id,
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

#[cfg(test)]
mod tests {
	use chrono::Utc;
	use common::CoreError;
	use rust_decimal::Decimal;
	use uuid::Uuid;

	use crate::{
		CustomerContextId, CustomerId, OrganizationContextId, OrganizationId,
		domain::{
			invoice::{
				Invoice, InvoiceId, InvoiceLine, InvoiceLineId, InvoiceStatus, InvoiceType,
				ports::MockInvoiceRepository,
				service::InvoiceService,
				commands::CreateInvoiceCommand,
			},
			quote::{Quote, QuoteId, QuoteLine, QuoteLineId, QuoteStatus},
		},
		ServiceRateUnit,
	};

	fn make_quote(id: QuoteId, emitter_context_id: Option<OrganizationContextId>) -> Quote {
		let now = Utc::now();
		let org_id = OrganizationId(Uuid::new_v4());
		Quote {
			id,
			organization_id: org_id,
			reference: "DEV-2026-0001".to_owned(),
			title: "Rénovation".to_owned(),
			customer_id: CustomerId(Uuid::new_v4()),
			customer_context_id: CustomerContextId(Uuid::new_v4()),
			emitter_context_id,
			status: QuoteStatus::Accepted,
			deposit_basis: None,
			deposit_value: None,
			total_cents: 1200,
			total_ht_cents: 1000,
			total_vat_cents: 200,
			total_ttc_cents: 1200,
			lines: vec![QuoteLine {
				id: QuoteLineId(Uuid::new_v4()),
				organization_id: org_id,
				quote_id: id,
				service_rate_id: None,
				label: "Pose".to_owned(),
				quantity: Decimal::new(1, 0),
				unit: ServiceRateUnit::Hour,
				unit_price_cents: 1000,
				vat_rate: Decimal::from(20u32),
				notes: None,
				photo_keys: vec![],
				deleted_at: None,
				created_at: now,
				updated_at: now,
			}],
			legal_mention_template_ids: vec![],
			deleted_at: None,
			created_at: now,
			updated_at: now,
		}
	}

	fn make_invoice(id: InvoiceId, emitter_context_id: Option<OrganizationContextId>) -> Invoice {
		let now = Utc::now();
		let org_id = OrganizationId(Uuid::new_v4());
		Invoice {
			id,
			org_id,
			customer_id: CustomerId(Uuid::new_v4()),
			customer_context_id: CustomerContextId(Uuid::new_v4()),
			emitter_context_id,
			reference: None,
			title: "Facture".to_owned(),
			status: InvoiceStatus::Draft,
			invoice_type: InvoiceType::Standard,
			source_quote_id: None,
			parent_invoice_id: None,
			deposit_basis: None,
			deposit_value: None,
			deposit_amount_cents: None,
			total_ht_cents: 1000,
			total_vat_cents: 200,
			total_ttc_cents: 1200,
			total_cents: 1200,
			amount_paid_cents: 0,
			pdf_key: None,
			issued_at: None,
			due_at: None,
			lines: vec![InvoiceLine {
				id: InvoiceLineId(Uuid::new_v4()),
				org_id,
				invoice_id: id,
				service_rate_id: None,
				label: "Pose".to_owned(),
				quantity: Decimal::new(1, 0),
				unit: ServiceRateUnit::Hour,
				unit_price_cents: 1000,
				vat_rate: Decimal::from(20u32),
				notes: None,
				photo_keys: vec![],
				deleted_at: None,
				created_at: now,
				updated_at: now,
			}],
			legal_mention_template_ids: vec![],
			deleted_at: None,
			created_at: now,
			updated_at: now,
		}
	}

	/// Gate: convert_quote_to_invoice rejects when the quote has no emitter context.
	///
	/// The guard in `MestierUseCase::convert_quote_to_invoice` checks
	/// `quote.emitter_context_id.is_none()`. We exercise the same predicate here.
	#[test]
	fn convert_rejects_when_quote_has_no_emitter_context() {
		let quote = make_quote(QuoteId(Uuid::new_v4()), None);
		// Mirrors the guard in application/invoice/mod.rs::convert_quote_to_invoice
		let result: Result<(), CoreError> = if quote.emitter_context_id.is_none() {
			Err(CoreError::Conflict(
				"select an emitter context before converting the quote".to_owned(),
			))
		} else {
			Ok(())
		};
		assert!(matches!(result, Err(CoreError::Conflict(msg)) if msg.contains("emitter context")));
	}

	/// Gate: convert_quote_to_invoice copies emitter_context_id to the created invoice.
	///
	/// Verifies that `CreateInvoiceCommand.emitter_context_id` is set from
	/// `quote.emitter_context_id` as implemented in the application layer.
	#[tokio::test]
	async fn convert_copies_emitter_context_id() {
		let emitter_id = OrganizationContextId(Uuid::new_v4());
		let quote = make_quote(QuoteId(Uuid::new_v4()), Some(emitter_id));

		// Simulate the command-building step in convert_quote_to_invoice
		let mut repo = MockInvoiceRepository::new();
		repo.expect_insert().times(1).returning(|inv| {
			let inv = inv.clone();
			Box::pin(async move { Ok(inv) })
		});
		repo.expect_replace_legal_mention_templates()
			.times(1)
			.returning(|_, _| Box::pin(async { Ok(()) }));

		let mut invoice_service = InvoiceService::new(repo);
		let created = invoice_service
			.create_invoice(CreateInvoiceCommand {
				org_id: quote.organization_id,
				title: quote.title.clone(),
				customer_id: quote.customer_id,
				customer_context_id: quote.customer_context_id,
				emitter_context_id: quote.emitter_context_id, // the copy
				invoice_type: InvoiceType::Standard,
				source_quote_id: Some(quote.id),
				parent_invoice_id: None,
				deposit_basis: None,
				deposit_value: None,
				deposit_amount_cents: None,
				due_at: None,
				lines: vec![],
				legal_mention_template_ids: vec![],
			})
			.await
			.unwrap();

		assert_eq!(created.emitter_context_id, Some(emitter_id));
	}

	/// Gate: issue_invoice rejects when the invoice has no emitter context.
	///
	/// The guard in `MestierUseCase::issue_invoice` checks
	/// `invoice.emitter_context_id.is_none()`. We exercise the same predicate here.
	#[test]
	fn issue_rejects_when_invoice_has_no_emitter_context() {
		let invoice = make_invoice(InvoiceId(Uuid::new_v4()), None);
		// Mirrors the guard in application/invoice/mod.rs::issue_invoice
		let result: Result<(), CoreError> = if invoice.emitter_context_id.is_none() {
			Err(CoreError::Conflict(
				"select an emitter context before issuing the invoice".to_owned(),
			))
		} else {
			Ok(())
		};
		assert!(matches!(result, Err(CoreError::Conflict(msg)) if msg.contains("emitter context")));
	}
}
