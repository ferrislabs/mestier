use chrono::{Datelike, Utc};
use common::{CoreError, generate_uuid_v7};
use rust_decimal::Decimal;

use crate::{
	OrganizationId, ServiceRateUnit,
	domain::{
		billing::{BillingLine, DepositBasis, compute_totals, remaining_to_pay, resolve_deposit},
		invoice::{
			Invoice, InvoiceId, InvoiceLine, InvoiceLineId, InvoiceStatus, InvoiceType,
			commands::{
				CreateInvoiceCommand, InvoiceLineCommand, UpdateInvoiceCommand,
				UpdateInvoiceStatusCommand,
			},
			ports::InvoiceRepository,
		},
	},
};

pub struct InvoiceService<R>
where
	R: InvoiceRepository,
{
	repo: R,
}

impl<R> InvoiceService<R>
where
	R: InvoiceRepository,
{
	pub fn new(repo: R) -> Self {
		Self { repo }
	}

	pub async fn create_invoice(
		&mut self,
		command: CreateInvoiceCommand,
	) -> Result<Invoice, CoreError> {
		let now = Utc::now();
		let invoice_id = InvoiceId(generate_uuid_v7());
		validate_title(&command.title)?;
		let lines = build_invoice_lines(command.org_id, invoice_id, command.lines, now)?;
		let (total_ht_cents, total_vat_cents, total_ttc_cents) = compute_invoice_totals(&lines)?;
		let total_cents = total_ttc_cents;

		let invoice = self
			.repo
			.insert(&Invoice {
				id: invoice_id,
				org_id: command.org_id,
				customer_id: command.customer_id,
				customer_context_id: command.customer_context_id,
				reference: None,
				title: command.title.trim().to_owned(),
				status: InvoiceStatus::Draft,
				invoice_type: command.invoice_type,
				source_quote_id: command.source_quote_id,
				parent_invoice_id: command.parent_invoice_id,
				deposit_basis: command.deposit_basis,
				deposit_value: command.deposit_value,
				deposit_amount_cents: command.deposit_amount_cents,
				total_ht_cents,
				total_vat_cents,
				total_ttc_cents,
				total_cents,
				amount_paid_cents: 0,
				pdf_key: None,
				issued_at: None,
				due_at: command.due_at,
				lines,
				legal_mention_template_ids: vec![],
				deleted_at: None,
				created_at: now,
				updated_at: now,
			})
			.await?;

		self.repo
			.replace_legal_mention_templates(&invoice, &command.legal_mention_template_ids)
			.await?;

		Ok(Invoice {
			legal_mention_template_ids: command.legal_mention_template_ids,
			..invoice
		})
	}

	pub async fn get_invoice(&mut self, id: InvoiceId) -> Result<Invoice, CoreError> {
		let invoice = self.repo.find_by_id(id).await?.ok_or(CoreError::NotFound)?;
		let legal_mention_template_ids =
			self.repo.find_legal_mention_template_ids(id).await?;
		Ok(Invoice {
			legal_mention_template_ids,
			..invoice
		})
	}

	pub async fn list_invoices(
		&mut self,
		org_id: OrganizationId,
		limit: u64,
		offset: u64,
	) -> Result<(Vec<Invoice>, u64), CoreError> {
		let (invoices, total) = self
			.repo
			.list_by_organization(org_id, limit, offset)
			.await?;

		let mut enriched = Vec::with_capacity(invoices.len());
		for invoice in invoices {
			let legal_mention_template_ids =
				self.repo.find_legal_mention_template_ids(invoice.id).await?;
			enriched.push(Invoice {
				legal_mention_template_ids,
				..invoice
			});
		}

		Ok((enriched, total))
	}

	pub async fn update_invoice(
		&mut self,
		command: UpdateInvoiceCommand,
	) -> Result<Invoice, CoreError> {
		let existing = self.get_invoice(command.id).await?;

		if existing.status != InvoiceStatus::Draft {
			return Err(CoreError::Conflict(
				"invoice can only be updated while in DRAFT status".to_owned(),
			));
		}

		let now = Utc::now();
		validate_title(&command.title)?;
		let lines =
			build_invoice_lines(existing.org_id, existing.id, command.lines, now)?;
		let (total_ht_cents, total_vat_cents, total_ttc_cents) = compute_invoice_totals(&lines)?;
		let total_cents = total_ttc_cents;

		let invoice = self
			.repo
			.update(&Invoice {
				id: existing.id,
				org_id: existing.org_id,
				customer_id: command.customer_id,
				customer_context_id: command.customer_context_id,
				reference: existing.reference,
				title: command.title.trim().to_owned(),
				status: existing.status,
				invoice_type: existing.invoice_type,
				source_quote_id: existing.source_quote_id,
				parent_invoice_id: existing.parent_invoice_id,
				deposit_basis: command.deposit_basis,
				deposit_value: command.deposit_value,
				deposit_amount_cents: command.deposit_amount_cents,
				total_ht_cents,
				total_vat_cents,
				total_ttc_cents,
				total_cents,
				amount_paid_cents: existing.amount_paid_cents,
				pdf_key: existing.pdf_key,
				issued_at: existing.issued_at,
				due_at: command.due_at,
				lines,
				legal_mention_template_ids: vec![],
				deleted_at: existing.deleted_at,
				created_at: existing.created_at,
				updated_at: now,
			})
			.await?;

		self.repo
			.replace_legal_mention_templates(&invoice, &command.legal_mention_template_ids)
			.await?;

		Ok(Invoice {
			legal_mention_template_ids: command.legal_mention_template_ids,
			..invoice
		})
	}

	pub async fn update_invoice_status(
		&mut self,
		command: UpdateInvoiceStatusCommand,
	) -> Result<Invoice, CoreError> {
		let existing = self.get_invoice(command.id).await?;
		let now = Utc::now();

		// Assign reference on first SENT transition, using FAC-YYYY-NNNN prefix.
		let (reference, issued_at) = if command.status == InvoiceStatus::Sent
			&& existing.reference.is_none()
		{
			let ref_str = self
				.repo
				.next_reference(existing.org_id, now.year())
				.await?;
			(Some(ref_str), Some(now))
		} else {
			(existing.reference, existing.issued_at)
		};

		let invoice = self
			.repo
			.update_status(command.id, command.status, reference, issued_at, now)
			.await?;

		let legal_mention_template_ids =
			self.repo.find_legal_mention_template_ids(command.id).await?;
		Ok(Invoice {
			legal_mention_template_ids,
			..invoice
		})
	}

	pub async fn soft_delete_invoice(&mut self, id: InvoiceId) -> Result<(), CoreError> {
		let existing = self.get_invoice(id).await?;

		if existing.status != InvoiceStatus::Draft {
			return Err(CoreError::Conflict(
				"invoice can only be deleted while in DRAFT status".to_owned(),
			));
		}

		self.repo.soft_delete(id, Utc::now()).await
	}

	pub async fn create_deposit_invoice(
		&mut self,
		parent_invoice_id: InvoiceId,
		basis: DepositBasis,
		value: Decimal,
	) -> Result<Invoice, CoreError> {
		let parent = self
			.repo
			.find_by_id(parent_invoice_id)
			.await?
			.filter(|inv| inv.deleted_at.is_none())
			.ok_or(CoreError::NotFound)?;

		let deposit_cents = resolve_deposit(parent.total_ttc_cents as i64, basis, value);

		let now = Utc::now();
		let invoice_id = InvoiceId(generate_uuid_v7());

		let deposit_basis_str = match basis {
			DepositBasis::Percent => "PERCENT",
			DepositBasis::Fixed => "FIXED",
		};

		// Proportional HT for the deposit; if TTC is zero, HT is zero.
		let total_ht_cents = if parent.total_ttc_cents > 0 {
			((parent.total_ht_cents as i64 * deposit_cents) / parent.total_ttc_cents as i64) as i32
		} else {
			0
		};
		let total_ttc_cents = deposit_cents as i32;
		let total_vat_cents = total_ttc_cents - total_ht_cents;
		let total_cents = total_ttc_cents;

		let line = InvoiceLine {
			id: InvoiceLineId(generate_uuid_v7()),
			org_id: parent.org_id,
			invoice_id,
			service_rate_id: None,
			label: "Acompte".to_owned(),
			quantity: Decimal::from(1u32),
			unit: ServiceRateUnit::Hour,
			unit_price_cents: deposit_cents as i32,
			vat_rate: Decimal::ZERO,
			notes: None,
			photo_keys: vec![],
			deleted_at: None,
			created_at: now,
			updated_at: now,
		};

		let invoice = self
			.repo
			.insert(&Invoice {
				id: invoice_id,
				org_id: parent.org_id,
				customer_id: parent.customer_id,
				customer_context_id: parent.customer_context_id,
				reference: None,
				title: format!("Acompte — {}", parent.title),
				status: InvoiceStatus::Draft,
				invoice_type: InvoiceType::Deposit,
				source_quote_id: parent.source_quote_id,
				parent_invoice_id: Some(parent.id),
				deposit_basis: Some(deposit_basis_str.to_owned()),
				deposit_value: Some(value),
				deposit_amount_cents: Some(deposit_cents as i32),
				total_ht_cents,
				total_vat_cents,
				total_ttc_cents,
				total_cents,
				amount_paid_cents: 0,
				pdf_key: None,
				issued_at: None,
				due_at: None,
				lines: vec![line],
				legal_mention_template_ids: vec![],
				deleted_at: None,
				created_at: now,
				updated_at: now,
			})
			.await?;

		self.repo
			.replace_legal_mention_templates(&invoice, &[])
			.await?;

		Ok(invoice)
	}

	pub async fn create_balance_invoice(
		&mut self,
		parent_invoice_id: InvoiceId,
	) -> Result<Invoice, CoreError> {
		let parent = self
			.repo
			.find_by_id(parent_invoice_id)
			.await?
			.filter(|inv| inv.deleted_at.is_none())
			.ok_or(CoreError::NotFound)?;

		let deposits = self
			.repo
			.list_deposits_for_parent(parent.id)
			.await?;

		let deposits_sum: i64 = deposits
			.iter()
			.map(|d| {
				d.deposit_amount_cents
					.unwrap_or(d.total_ttc_cents) as i64
			})
			.sum();

		let remaining = remaining_to_pay(parent.total_ttc_cents as i64, deposits_sum);

		if remaining <= 0 {
			return Err(CoreError::Conflict(
				"balance invoice cannot be created: remaining amount to pay is zero or negative"
					.to_owned(),
			));
		}

		let now = Utc::now();
		let invoice_id = InvoiceId(generate_uuid_v7());

		// Clone parent lines with fresh IDs bound to the new invoice.
		// Simplification: total_ht_cents and total_vat_cents reflect the parent's full
		// breakdown; the amount actually due is `remaining` (stored in total_ttc_cents /
		// total_cents), with deposit_amount_cents recording the already-deducted sum.
		let lines: Vec<InvoiceLine> = parent
			.lines
			.iter()
			.filter(|l| l.deleted_at.is_none())
			.map(|l| InvoiceLine {
				id: InvoiceLineId(generate_uuid_v7()),
				org_id: l.org_id,
				invoice_id,
				service_rate_id: l.service_rate_id,
				label: l.label.clone(),
				quantity: l.quantity,
				unit: l.unit,
				unit_price_cents: l.unit_price_cents,
				vat_rate: l.vat_rate,
				notes: l.notes.clone(),
				photo_keys: l.photo_keys.clone(),
				deleted_at: None,
				created_at: now,
				updated_at: now,
			})
			.collect();

		let invoice = self
			.repo
			.insert(&Invoice {
				id: invoice_id,
				org_id: parent.org_id,
				customer_id: parent.customer_id,
				customer_context_id: parent.customer_context_id,
				reference: None,
				title: format!("Solde — {}", parent.title),
				status: InvoiceStatus::Draft,
				invoice_type: InvoiceType::Balance,
				source_quote_id: parent.source_quote_id,
				parent_invoice_id: Some(parent.id),
				deposit_basis: None,
				deposit_value: None,
				deposit_amount_cents: Some(deposits_sum as i32),
				total_ht_cents: parent.total_ht_cents,
				total_vat_cents: parent.total_vat_cents,
				total_ttc_cents: remaining as i32,
				total_cents: remaining as i32,
				amount_paid_cents: 0,
				pdf_key: None,
				issued_at: None,
				due_at: None,
				lines,
				legal_mention_template_ids: vec![],
				deleted_at: None,
				created_at: now,
				updated_at: now,
			})
			.await?;

		self.repo
			.replace_legal_mention_templates(&invoice, &[])
			.await?;

		Ok(invoice)
	}
}

fn validate_title(title: &str) -> Result<(), CoreError> {
	if title.trim().is_empty() {
		return Err(CoreError::Conflict(
			"invoice title cannot be empty".to_owned(),
		));
	}
	Ok(())
}

fn build_invoice_lines(
	org_id: OrganizationId,
	invoice_id: InvoiceId,
	commands: Vec<InvoiceLineCommand>,
	now: chrono::DateTime<Utc>,
) -> Result<Vec<InvoiceLine>, CoreError> {
	commands
		.into_iter()
		.map(|command| build_invoice_line(org_id, invoice_id, command, now))
		.collect()
}

fn build_invoice_line(
	org_id: OrganizationId,
	invoice_id: InvoiceId,
	command: InvoiceLineCommand,
	now: chrono::DateTime<Utc>,
) -> Result<InvoiceLine, CoreError> {
	validate_line(&command)?;

	Ok(InvoiceLine {
		id: InvoiceLineId(generate_uuid_v7()),
		org_id,
		invoice_id,
		service_rate_id: command.service_rate_id,
		label: command.label,
		quantity: command.quantity,
		unit: command.unit,
		unit_price_cents: command.unit_price_cents,
		vat_rate: command.vat_rate,
		notes: command.notes,
		photo_keys: command.photo_keys,
		deleted_at: None,
		created_at: now,
		updated_at: now,
	})
}

fn validate_line(command: &InvoiceLineCommand) -> Result<(), CoreError> {
	if command.label.trim().is_empty() {
		return Err(CoreError::Conflict(
			"invoice line label cannot be empty".to_owned(),
		));
	}

	if command.quantity <= Decimal::ZERO {
		return Err(CoreError::Conflict(
			"invoice line quantity must be positive".to_owned(),
		));
	}

	if command.unit_price_cents < 0 {
		return Err(CoreError::Conflict(
			"invoice line unit price cannot be negative".to_owned(),
		));
	}

	if command
		.notes
		.as_ref()
		.is_some_and(|notes| notes.trim().is_empty())
	{
		return Err(CoreError::Conflict(
			"invoice line notes cannot be empty when present".to_owned(),
		));
	}

	if command.photo_keys.iter().any(|key| key.trim().is_empty()) {
		return Err(CoreError::Conflict(
			"invoice line photo keys cannot be empty".to_owned(),
		));
	}

	Ok(())
}

fn compute_invoice_totals(lines: &[InvoiceLine]) -> Result<(i32, i32, i32), CoreError> {
	let billing_lines: Vec<BillingLine> = lines
		.iter()
		.map(|line| BillingLine {
			quantity: line.quantity,
			unit_price_cents: line.unit_price_cents as i64,
			vat_rate: line.vat_rate,
		})
		.collect();

	let totals = compute_totals(&billing_lines);

	let total_ht_cents = i32::try_from(totals.total_ht_cents).map_err(|_| {
		CoreError::Conflict("invoice HT total is outside supported bounds".to_owned())
	})?;
	let total_vat_cents = i32::try_from(totals.total_vat_cents).map_err(|_| {
		CoreError::Conflict("invoice VAT total is outside supported bounds".to_owned())
	})?;
	let total_ttc_cents = i32::try_from(totals.total_ttc_cents).map_err(|_| {
		CoreError::Conflict("invoice TTC total is outside supported bounds".to_owned())
	})?;

	Ok((total_ht_cents, total_vat_cents, total_ttc_cents))
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::{
		CustomerContextId, CustomerId, LegalMentionTemplateId, ServiceRateUnit,
		domain::invoice::{InvoiceType, ports::MockInvoiceRepository},
	};
	use mockall::predicate::eq;
	use rust_decimal::Decimal;
	use uuid::Uuid;

	fn line_command(quantity: Decimal, unit_price_cents: i32) -> InvoiceLineCommand {
		InvoiceLineCommand {
			service_rate_id: None,
			label: "Pose carrelage".to_owned(),
			quantity,
			unit: ServiceRateUnit::M2,
			unit_price_cents,
			vat_rate: Decimal::from(20u32),
			notes: None,
			photo_keys: vec![],
		}
	}

	fn invoice(id: InvoiceId) -> Invoice {
		let now = Utc::now();
		let org_id = OrganizationId(Uuid::new_v4());
		Invoice {
			id,
			org_id,
			customer_id: CustomerId(Uuid::new_v4()),
			customer_context_id: CustomerContextId(Uuid::new_v4()),
			reference: None,
			title: "Facture chantier".to_owned(),
			status: InvoiceStatus::Draft,
			invoice_type: InvoiceType::Standard,
			source_quote_id: None,
			parent_invoice_id: None,
			deposit_basis: None,
			deposit_value: None,
			deposit_amount_cents: None,
			total_ht_cents: 5000,
			total_vat_cents: 1000,
			total_ttc_cents: 6000,
			total_cents: 6000,
			amount_paid_cents: 0,
			pdf_key: None,
			issued_at: None,
			due_at: None,
			lines: vec![InvoiceLine {
				id: InvoiceLineId(Uuid::new_v4()),
				org_id,
				invoice_id: id,
				service_rate_id: None,
				label: "Pose carrelage".to_owned(),
				quantity: Decimal::new(1, 0),
				unit: ServiceRateUnit::M2,
				unit_price_cents: 5000,
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

	// ---- create ----

	#[tokio::test]
	async fn create_invoice_computes_totals_from_lines() {
		let mut repo = MockInvoiceRepository::new();
		repo.expect_insert().times(1).returning(|inv| {
			let inv = inv.clone();
			Box::pin(async move { Ok(inv) })
		});
		repo.expect_replace_legal_mention_templates()
			.times(1)
			.returning(|_, _| Box::pin(async { Ok(()) }));

		let mut service = InvoiceService::new(repo);
		let created = service
			.create_invoice(CreateInvoiceCommand {
				org_id: OrganizationId(Uuid::new_v4()),
				title: "Facture cuisine".to_owned(),
				customer_id: CustomerId(Uuid::new_v4()),
				customer_context_id: CustomerContextId(Uuid::new_v4()),
				invoice_type: InvoiceType::Standard,
				source_quote_id: None,
				parent_invoice_id: None,
				deposit_basis: None,
				deposit_value: None,
				deposit_amount_cents: None,
				due_at: None,
				lines: vec![
					line_command(Decimal::new(25, 1), 1200),
					line_command(Decimal::new(1, 0), 500),
				],
				legal_mention_template_ids: vec![],
			})
			.await
			.unwrap();

		assert_eq!(created.status, InvoiceStatus::Draft);
		assert_eq!(created.reference, None);
		// 2.5*1200=3000 + 1*500=500 → HT=3500, VAT@20%=700, TTC=4200
		assert_eq!(created.total_ht_cents, 3500);
		assert_eq!(created.total_vat_cents, 700);
		assert_eq!(created.total_ttc_cents, 4200);
		assert_eq!(created.total_cents, 4200);
	}

	#[tokio::test]
	async fn create_invoice_has_no_reference_until_sent() {
		let mut repo = MockInvoiceRepository::new();
		repo.expect_insert().times(1).returning(|inv| {
			let inv = inv.clone();
			Box::pin(async move { Ok(inv) })
		});
		repo.expect_replace_legal_mention_templates()
			.times(1)
			.returning(|_, _| Box::pin(async { Ok(()) }));

		let mut service = InvoiceService::new(repo);
		let created = service
			.create_invoice(CreateInvoiceCommand {
				org_id: OrganizationId(Uuid::new_v4()),
				title: "Facture test".to_owned(),
				customer_id: CustomerId(Uuid::new_v4()),
				customer_context_id: CustomerContextId(Uuid::new_v4()),
				invoice_type: InvoiceType::Standard,
				source_quote_id: None,
				parent_invoice_id: None,
				deposit_basis: None,
				deposit_value: None,
				deposit_amount_cents: None,
				due_at: None,
				lines: vec![line_command(Decimal::new(1, 0), 1000)],
				legal_mention_template_ids: vec![],
			})
			.await
			.unwrap();

		assert_eq!(created.reference, None);
		assert_eq!(created.issued_at, None);
	}

	// ---- update ----

	#[tokio::test]
	async fn update_invoice_recomputes_totals() {
		let id = InvoiceId(Uuid::new_v4());
		let mut repo = MockInvoiceRepository::new();
		repo.expect_find_by_id()
			.with(eq(id))
			.returning(move |_| Box::pin(async move { Ok(Some(invoice(id))) }));
		repo.expect_find_legal_mention_template_ids()
			.with(eq(id))
			.times(1)
			.returning(|_| Box::pin(async { Ok(vec![]) }));
		repo.expect_update().times(1).returning(|inv| {
			let inv = inv.clone();
			Box::pin(async move { Ok(inv) })
		});
		repo.expect_replace_legal_mention_templates()
			.times(1)
			.returning(|_, _| Box::pin(async { Ok(()) }));

		let mut service = InvoiceService::new(repo);
		let updated = service
			.update_invoice(UpdateInvoiceCommand {
				id,
				title: "Facture mise à jour".to_owned(),
				customer_id: CustomerId(Uuid::new_v4()),
				customer_context_id: CustomerContextId(Uuid::new_v4()),
				deposit_basis: None,
				deposit_value: None,
				deposit_amount_cents: None,
				due_at: None,
				lines: vec![line_command(Decimal::new(3, 0), 2000)],
				legal_mention_template_ids: vec![],
			})
			.await
			.unwrap();

		// 3*2000=6000 HT, VAT@20%=1200, TTC=7200
		assert_eq!(updated.total_ht_cents, 6000);
		assert_eq!(updated.total_vat_cents, 1200);
		assert_eq!(updated.total_ttc_cents, 7200);
		assert_eq!(updated.total_cents, 7200);
	}

	#[tokio::test]
	async fn update_invoice_rejected_when_not_draft() {
		let id = InvoiceId(Uuid::new_v4());
		let mut repo = MockInvoiceRepository::new();
		repo.expect_find_by_id()
			.with(eq(id))
			.returning(move |_| {
				let mut inv = invoice(id);
				inv.status = InvoiceStatus::Sent;
				Box::pin(async move { Ok(Some(inv)) })
			});
		repo.expect_find_legal_mention_template_ids()
			.with(eq(id))
			.times(1)
			.returning(|_| Box::pin(async { Ok(vec![]) }));

		let mut service = InvoiceService::new(repo);
		let result = service
			.update_invoice(UpdateInvoiceCommand {
				id,
				title: "Tentative modif".to_owned(),
				customer_id: CustomerId(Uuid::new_v4()),
				customer_context_id: CustomerContextId(Uuid::new_v4()),
				deposit_basis: None,
				deposit_value: None,
				deposit_amount_cents: None,
				due_at: None,
				lines: vec![line_command(Decimal::new(1, 0), 1000)],
				legal_mention_template_ids: vec![],
			})
			.await;

		assert!(matches!(result, Err(CoreError::Conflict(_))));
	}

	// ---- status transition ----

	#[tokio::test]
	async fn update_status_to_sent_assigns_reference_and_issued_at() {
		let id = InvoiceId(Uuid::new_v4());
		let org_id = OrganizationId(Uuid::new_v4());
		let mut inv = invoice(id);
		inv.org_id = org_id;

		let mut repo = MockInvoiceRepository::new();
		repo.expect_find_by_id()
			.with(eq(id))
			.returning(move |_| {
				let inv = inv.clone();
				Box::pin(async move { Ok(Some(inv)) })
			});
		repo.expect_find_legal_mention_template_ids()
			.with(eq(id))
			.times(2)
			.returning(|_| Box::pin(async { Ok(vec![]) }));
		repo.expect_next_reference()
			.times(1)
			.returning(|_, year| {
				Box::pin(async move { Ok(format!("FAC-{year}-0001")) })
			});
		repo.expect_update_status()
			.times(1)
			.returning(move |_, _, ref_opt, issued_opt, _| {
				let mut inv2 = invoice(id);
				inv2.status = InvoiceStatus::Sent;
				inv2.reference = ref_opt;
				inv2.issued_at = issued_opt;
				Box::pin(async move { Ok(inv2) })
			});

		let mut service = InvoiceService::new(repo);
		let updated = service
			.update_invoice_status(UpdateInvoiceStatusCommand {
				id,
				status: InvoiceStatus::Sent,
			})
			.await
			.unwrap();

		assert_eq!(updated.status, InvoiceStatus::Sent);
		let ref_str = updated.reference.as_deref().unwrap_or("");
		assert!(
			ref_str.starts_with("FAC-") && ref_str.contains("-0001"),
			"expected FAC-YYYY-0001, got {ref_str}"
		);
		assert!(updated.issued_at.is_some());
	}

	#[tokio::test]
	async fn update_status_to_sent_does_not_reassign_existing_reference() {
		let id = InvoiceId(Uuid::new_v4());
		let mut inv = invoice(id);
		inv.status = InvoiceStatus::Sent;
		inv.reference = Some("FAC-2026-0001".to_owned());
		inv.issued_at = Some(Utc::now());

		let mut repo = MockInvoiceRepository::new();
		repo.expect_find_by_id()
			.with(eq(id))
			.returning(move |_| {
				let inv = inv.clone();
				Box::pin(async move { Ok(Some(inv)) })
			});
		repo.expect_find_legal_mention_template_ids()
			.with(eq(id))
			.times(2)
			.returning(|_| Box::pin(async { Ok(vec![]) }));
		// next_reference must NOT be called
		repo.expect_update_status()
			.times(1)
			.returning(move |_, _, ref_opt, issued_opt, _| {
				let mut inv2 = invoice(id);
				inv2.status = InvoiceStatus::Paid;
				inv2.reference = ref_opt;
				inv2.issued_at = issued_opt;
				Box::pin(async move { Ok(inv2) })
			});

		let mut service = InvoiceService::new(repo);
		let updated = service
			.update_invoice_status(UpdateInvoiceStatusCommand {
				id,
				status: InvoiceStatus::Paid,
			})
			.await
			.unwrap();

		assert_eq!(updated.status, InvoiceStatus::Paid);
		assert_eq!(updated.reference.as_deref(), Some("FAC-2026-0001"));
	}

	// ---- soft delete ----

	#[tokio::test]
	async fn soft_delete_invoice_succeeds_when_draft() {
		let id = InvoiceId(Uuid::new_v4());
		let mut repo = MockInvoiceRepository::new();
		repo.expect_find_by_id()
			.with(eq(id))
			.returning(move |_| Box::pin(async move { Ok(Some(invoice(id))) }));
		repo.expect_find_legal_mention_template_ids()
			.with(eq(id))
			.times(1)
			.returning(|_| Box::pin(async { Ok(vec![]) }));
		repo.expect_soft_delete()
			.withf(move |deleted_id, _| *deleted_id == id)
			.returning(|_, _| Box::pin(async { Ok(()) }));

		let mut service = InvoiceService::new(repo);
		service.soft_delete_invoice(id).await.unwrap();
	}

	#[tokio::test]
	async fn soft_delete_invoice_rejected_when_not_draft() {
		let id = InvoiceId(Uuid::new_v4());
		let mut repo = MockInvoiceRepository::new();
		repo.expect_find_by_id()
			.with(eq(id))
			.returning(move |_| {
				let mut inv = invoice(id);
				inv.status = InvoiceStatus::Sent;
				Box::pin(async move { Ok(Some(inv)) })
			});
		repo.expect_find_legal_mention_template_ids()
			.with(eq(id))
			.times(1)
			.returning(|_| Box::pin(async { Ok(vec![]) }));

		let mut service = InvoiceService::new(repo);
		let result = service.soft_delete_invoice(id).await;

		assert!(matches!(result, Err(CoreError::Conflict(_))));
	}

	// ---- numbering ----

	#[tokio::test]
	async fn numbering_format_is_fac_yyyy_nnnn() {
		let id = InvoiceId(Uuid::new_v4());
		let org_id = OrganizationId(Uuid::new_v4());
		let mut inv = invoice(id);
		inv.org_id = org_id;

		let mut repo = MockInvoiceRepository::new();
		repo.expect_find_by_id()
			.with(eq(id))
			.returning(move |_| {
				let inv = inv.clone();
				Box::pin(async move { Ok(Some(inv)) })
			});
		repo.expect_find_legal_mention_template_ids()
			.with(eq(id))
			.times(2)
			.returning(|_| Box::pin(async { Ok(vec![]) }));
		repo.expect_next_reference()
			.times(1)
			.returning(|_, year| {
				Box::pin(async move { Ok(format!("FAC-{year}-0042")) })
			});
		repo.expect_update_status()
			.times(1)
			.returning(move |_, _, ref_opt, issued_opt, _| {
				let mut inv2 = invoice(id);
				inv2.status = InvoiceStatus::Sent;
				inv2.reference = ref_opt;
				inv2.issued_at = issued_opt;
				Box::pin(async move { Ok(inv2) })
			});

		let mut service = InvoiceService::new(repo);
		let updated = service
			.update_invoice_status(UpdateInvoiceStatusCommand {
				id,
				status: InvoiceStatus::Sent,
			})
			.await
			.unwrap();

		let ref_str = updated.reference.unwrap();
		let year = Utc::now().year();
		assert_eq!(ref_str, format!("FAC-{year}-0042"));
	}

	// ---- legal mentions ----

	#[tokio::test]
	async fn create_invoice_persists_legal_mention_template_ids() {
		let template_a = LegalMentionTemplateId(Uuid::new_v4());
		let template_b = LegalMentionTemplateId(Uuid::new_v4());
		let template_ids = vec![template_a, template_b];
		let template_ids_clone = template_ids.clone();

		let mut repo = MockInvoiceRepository::new();
		repo.expect_insert().times(1).returning(|inv| {
			let inv = inv.clone();
			Box::pin(async move { Ok(inv) })
		});
		repo.expect_replace_legal_mention_templates()
			.times(1)
			.returning(|_, _| Box::pin(async { Ok(()) }));

		let mut service = InvoiceService::new(repo);
		let created = service
			.create_invoice(CreateInvoiceCommand {
				org_id: OrganizationId(Uuid::new_v4()),
				title: "Facture avec mentions".to_owned(),
				customer_id: CustomerId(Uuid::new_v4()),
				customer_context_id: CustomerContextId(Uuid::new_v4()),
				invoice_type: InvoiceType::Standard,
				source_quote_id: None,
				parent_invoice_id: None,
				deposit_basis: None,
				deposit_value: None,
				deposit_amount_cents: None,
				due_at: None,
				lines: vec![line_command(Decimal::new(1, 0), 1000)],
				legal_mention_template_ids: template_ids_clone,
			})
			.await
			.unwrap();

		assert_eq!(created.legal_mention_template_ids, template_ids);
	}

	// ---- deposit invoice ----

	fn parent_invoice_120k(id: InvoiceId) -> Invoice {
		let now = Utc::now();
		let org_id = OrganizationId(Uuid::new_v4());
		Invoice {
			id,
			org_id,
			customer_id: CustomerId(Uuid::new_v4()),
			customer_context_id: CustomerContextId(Uuid::new_v4()),
			reference: None,
			title: "Chantier cuisine".to_owned(),
			status: InvoiceStatus::Draft,
			invoice_type: InvoiceType::Standard,
			source_quote_id: None,
			parent_invoice_id: None,
			deposit_basis: None,
			deposit_value: None,
			deposit_amount_cents: None,
			// 100 000 HT + 20 000 TVA = 120 000 TTC
			total_ht_cents: 100_000,
			total_vat_cents: 20_000,
			total_ttc_cents: 120_000,
			total_cents: 120_000,
			amount_paid_cents: 0,
			pdf_key: None,
			issued_at: None,
			due_at: None,
			lines: vec![InvoiceLine {
				id: InvoiceLineId(Uuid::new_v4()),
				org_id,
				invoice_id: id,
				service_rate_id: None,
				label: "Cuisine complète".to_owned(),
				quantity: Decimal::new(1, 0),
				unit: ServiceRateUnit::M2,
				unit_price_cents: 100_000,
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

	#[tokio::test]
	async fn deposit_30_percent_of_120k_ttc() {
		let parent_id = InvoiceId(Uuid::new_v4());
		let parent = parent_invoice_120k(parent_id);

		let mut repo = MockInvoiceRepository::new();
		repo.expect_find_by_id()
			.with(eq(parent_id))
			.times(1)
			.returning(move |_| {
				let p = parent.clone();
				Box::pin(async move { Ok(Some(p)) })
			});
		repo.expect_insert().times(1).returning(|inv| {
			let inv = inv.clone();
			Box::pin(async move { Ok(inv) })
		});
		repo.expect_replace_legal_mention_templates()
			.times(1)
			.returning(|_, _| Box::pin(async { Ok(()) }));

		let mut service = InvoiceService::new(repo);
		let deposit = service
			.create_deposit_invoice(
				parent_id,
				crate::domain::billing::DepositBasis::Percent,
				Decimal::from(30u32),
			)
			.await
			.unwrap();

		assert_eq!(deposit.total_ttc_cents, 36_000);
		assert_eq!(deposit.invoice_type, InvoiceType::Deposit);
		assert_eq!(deposit.parent_invoice_id, Some(parent_id));
		assert_eq!(deposit.deposit_basis.as_deref(), Some("PERCENT"));
		assert_eq!(deposit.deposit_amount_cents, Some(36_000));
	}

	#[tokio::test]
	async fn deposit_fixed_amount() {
		let parent_id = InvoiceId(Uuid::new_v4());
		let parent = parent_invoice_120k(parent_id);

		let mut repo = MockInvoiceRepository::new();
		repo.expect_find_by_id()
			.with(eq(parent_id))
			.times(1)
			.returning(move |_| {
				let p = parent.clone();
				Box::pin(async move { Ok(Some(p)) })
			});
		repo.expect_insert().times(1).returning(|inv| {
			let inv = inv.clone();
			Box::pin(async move { Ok(inv) })
		});
		repo.expect_replace_legal_mention_templates()
			.times(1)
			.returning(|_, _| Box::pin(async { Ok(()) }));

		let mut service = InvoiceService::new(repo);
		let deposit = service
			.create_deposit_invoice(
				parent_id,
				crate::domain::billing::DepositBasis::Fixed,
				Decimal::from(50_000u32),
			)
			.await
			.unwrap();

		assert_eq!(deposit.total_ttc_cents, 50_000);
		assert_eq!(deposit.deposit_basis.as_deref(), Some("FIXED"));
		assert_eq!(deposit.deposit_amount_cents, Some(50_000));
		assert_eq!(deposit.parent_invoice_id, Some(parent_id));
	}

	// ---- balance invoice ----

	#[tokio::test]
	async fn balance_after_36k_deposit_gives_84k_remaining() {
		let parent_id = InvoiceId(Uuid::new_v4());
		let parent = parent_invoice_120k(parent_id);
		let deposit_invoice_id = InvoiceId(Uuid::new_v4());
		let now = Utc::now();

		// A deposit invoice that already exists against the parent.
		let deposit_inv = Invoice {
			id: deposit_invoice_id,
			org_id: parent.org_id,
			customer_id: parent.customer_id,
			customer_context_id: parent.customer_context_id,
			reference: None,
			title: "Acompte — Chantier cuisine".to_owned(),
			status: InvoiceStatus::Draft,
			invoice_type: InvoiceType::Deposit,
			source_quote_id: None,
			parent_invoice_id: Some(parent_id),
			deposit_basis: Some("PERCENT".to_owned()),
			deposit_value: Some(Decimal::from(30u32)),
			deposit_amount_cents: Some(36_000),
			total_ht_cents: 30_000,
			total_vat_cents: 6_000,
			total_ttc_cents: 36_000,
			total_cents: 36_000,
			amount_paid_cents: 0,
			pdf_key: None,
			issued_at: None,
			due_at: None,
			lines: vec![],
			legal_mention_template_ids: vec![],
			deleted_at: None,
			created_at: now,
			updated_at: now,
		};

		let mut repo = MockInvoiceRepository::new();
		repo.expect_find_by_id()
			.with(eq(parent_id))
			.times(1)
			.returning(move |_| {
				let p = parent.clone();
				Box::pin(async move { Ok(Some(p)) })
			});
		repo.expect_list_deposits_for_parent()
			.with(eq(parent_id))
			.times(1)
			.returning(move |_| {
				let d = deposit_inv.clone();
				Box::pin(async move { Ok(vec![d]) })
			});
		repo.expect_insert().times(1).returning(|inv| {
			let inv = inv.clone();
			Box::pin(async move { Ok(inv) })
		});
		repo.expect_replace_legal_mention_templates()
			.times(1)
			.returning(|_, _| Box::pin(async { Ok(()) }));

		let mut service = InvoiceService::new(repo);
		let balance = service.create_balance_invoice(parent_id).await.unwrap();

		assert_eq!(balance.total_ttc_cents, 84_000);
		assert_eq!(balance.total_cents, 84_000);
		assert_eq!(balance.invoice_type, InvoiceType::Balance);
		assert_eq!(balance.parent_invoice_id, Some(parent_id));
		assert_eq!(balance.deposit_amount_cents, Some(36_000));
	}

	#[tokio::test]
	async fn balance_rejected_when_remaining_is_zero() {
		let parent_id = InvoiceId(Uuid::new_v4());
		let parent = parent_invoice_120k(parent_id);
		let now = Utc::now();

		// A deposit invoice that covers 100% of the TTC.
		let full_deposit = Invoice {
			id: InvoiceId(Uuid::new_v4()),
			org_id: parent.org_id,
			customer_id: parent.customer_id,
			customer_context_id: parent.customer_context_id,
			reference: None,
			title: "Acompte — Chantier cuisine".to_owned(),
			status: InvoiceStatus::Draft,
			invoice_type: InvoiceType::Deposit,
			source_quote_id: None,
			parent_invoice_id: Some(parent_id),
			deposit_basis: Some("PERCENT".to_owned()),
			deposit_value: Some(Decimal::from(100u32)),
			deposit_amount_cents: Some(120_000),
			total_ht_cents: 100_000,
			total_vat_cents: 20_000,
			total_ttc_cents: 120_000,
			total_cents: 120_000,
			amount_paid_cents: 0,
			pdf_key: None,
			issued_at: None,
			due_at: None,
			lines: vec![],
			legal_mention_template_ids: vec![],
			deleted_at: None,
			created_at: now,
			updated_at: now,
		};

		let mut repo = MockInvoiceRepository::new();
		repo.expect_find_by_id()
			.with(eq(parent_id))
			.times(1)
			.returning(move |_| {
				let p = parent.clone();
				Box::pin(async move { Ok(Some(p)) })
			});
		repo.expect_list_deposits_for_parent()
			.with(eq(parent_id))
			.times(1)
			.returning(move |_| {
				let d = full_deposit.clone();
				Box::pin(async move { Ok(vec![d]) })
			});

		let mut service = InvoiceService::new(repo);
		let result = service.create_balance_invoice(parent_id).await;

		assert!(matches!(result, Err(CoreError::Conflict(_))));
	}

	#[tokio::test]
	async fn deposit_and_balance_not_found_when_parent_missing() {
		let parent_id = InvoiceId(Uuid::new_v4());

		let mut repo = MockInvoiceRepository::new();
		repo.expect_find_by_id()
			.returning(|_| Box::pin(async { Ok(None) }));

		let mut service = InvoiceService::new(repo);
		let r1 = service
			.create_deposit_invoice(
				parent_id,
				crate::domain::billing::DepositBasis::Percent,
				Decimal::from(30u32),
			)
			.await;
		assert!(matches!(r1, Err(CoreError::NotFound)));

		// Need a fresh repo for the balance call because find_by_id can only be set once
		// with `.returning` (any call). A second mock is cleaner.
		let mut repo2 = MockInvoiceRepository::new();
		repo2
			.expect_find_by_id()
			.returning(|_| Box::pin(async { Ok(None) }));
		let mut service2 = InvoiceService::new(repo2);
		let r2 = service2.create_balance_invoice(parent_id).await;
		assert!(matches!(r2, Err(CoreError::NotFound)));
	}

	// ---- list ----

	#[tokio::test]
	async fn list_invoices_delegates_to_repo() {
		let org_id = OrganizationId(Uuid::new_v4());
		let invoice_id = InvoiceId(Uuid::new_v4());
		let mut repo = MockInvoiceRepository::new();
		repo.expect_list_by_organization()
			.with(eq(org_id), eq(25u64), eq(50u64))
			.returning(move |_, _, _| {
				Box::pin(async move { Ok((vec![invoice(invoice_id)], 1)) })
			});
		repo.expect_find_legal_mention_template_ids()
			.with(eq(invoice_id))
			.times(1)
			.returning(|_| Box::pin(async { Ok(vec![]) }));

		let mut service = InvoiceService::new(repo);
		let (items, total) = service.list_invoices(org_id, 25, 50).await.unwrap();

		assert_eq!(items.len(), 1);
		assert_eq!(total, 1);
	}

	// ---- input validation ----

	#[tokio::test]
	async fn rejects_empty_invoice_title() {
		let repo = MockInvoiceRepository::new();
		let mut service = InvoiceService::new(repo);
		let result = service
			.create_invoice(CreateInvoiceCommand {
				org_id: OrganizationId(Uuid::new_v4()),
				title: " ".to_owned(),
				customer_id: CustomerId(Uuid::new_v4()),
				customer_context_id: CustomerContextId(Uuid::new_v4()),
				invoice_type: InvoiceType::Standard,
				source_quote_id: None,
				parent_invoice_id: None,
				deposit_basis: None,
				deposit_value: None,
				deposit_amount_cents: None,
				due_at: None,
				lines: vec![line_command(Decimal::new(1, 0), 1000)],
				legal_mention_template_ids: vec![],
			})
			.await;

		assert!(matches!(result, Err(CoreError::Conflict(_))));
	}

	#[tokio::test]
	async fn rejects_invalid_line_quantity() {
		let repo = MockInvoiceRepository::new();
		let mut service = InvoiceService::new(repo);
		let result = service
			.create_invoice(CreateInvoiceCommand {
				org_id: OrganizationId(Uuid::new_v4()),
				title: "Facture test".to_owned(),
				customer_id: CustomerId(Uuid::new_v4()),
				customer_context_id: CustomerContextId(Uuid::new_v4()),
				invoice_type: InvoiceType::Standard,
				source_quote_id: None,
				parent_invoice_id: None,
				deposit_basis: None,
				deposit_value: None,
				deposit_amount_cents: None,
				due_at: None,
				lines: vec![line_command(Decimal::ZERO, 1000)],
				legal_mention_template_ids: vec![],
			})
			.await;

		assert!(matches!(result, Err(CoreError::Conflict(_))));
	}

	// ---- convert quote to invoice ----

	#[tokio::test]
	async fn create_invoice_from_quote_sets_source_quote_id_and_copies_lines() {
		use crate::domain::quote::QuoteId;

		let quote_id = QuoteId(Uuid::new_v4());
		let template_a = LegalMentionTemplateId(Uuid::new_v4());
		let template_ids = vec![template_a];
		let template_ids_clone = template_ids.clone();

		let mut repo = MockInvoiceRepository::new();
		repo.expect_insert().times(1).returning(|inv| {
			let inv = inv.clone();
			Box::pin(async move { Ok(inv) })
		});
		repo.expect_replace_legal_mention_templates()
			.times(1)
			.returning(|_, _| Box::pin(async { Ok(()) }));

		let mut service = InvoiceService::new(repo);
		let created = service
			.create_invoice(CreateInvoiceCommand {
				org_id: OrganizationId(Uuid::new_v4()),
				title: "Rénovation cuisine".to_owned(),
				customer_id: CustomerId(Uuid::new_v4()),
				customer_context_id: CustomerContextId(Uuid::new_v4()),
				invoice_type: InvoiceType::Standard,
				source_quote_id: Some(quote_id),
				parent_invoice_id: None,
				deposit_basis: None,
				deposit_value: None,
				deposit_amount_cents: None,
				due_at: None,
				lines: vec![
					line_command(Decimal::new(2, 0), 3000),
					line_command(Decimal::new(1, 0), 500),
				],
				legal_mention_template_ids: template_ids_clone,
			})
			.await
			.unwrap();

		// source_quote_id must be forwarded
		assert_eq!(created.source_quote_id, Some(quote_id));
		// invoice_type is Standard
		assert_eq!(created.invoice_type, InvoiceType::Standard);
		// starts as DRAFT with no reference
		assert_eq!(created.status, InvoiceStatus::Draft);
		assert_eq!(created.reference, None);
		// two lines were copied
		assert_eq!(created.lines.len(), 2);
		// totals are recomputed from lines: 2*3000=6000 + 1*500=500 → HT=6500, VAT@20%=1300, TTC=7800
		assert_eq!(created.total_ht_cents, 6500);
		assert_eq!(created.total_vat_cents, 1300);
		assert_eq!(created.total_ttc_cents, 7800);
		assert_eq!(created.total_cents, 7800);
		// legal mention template IDs are preserved
		assert_eq!(created.legal_mention_template_ids, template_ids);
	}
}
