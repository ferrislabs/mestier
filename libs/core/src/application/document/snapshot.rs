use chrono::{DateTime, Utc};
use rust_decimal::Decimal;

use crate::{
	BillingSettings, Customer, CustomerContext, OrganizationContext,
	domain::{
		billing::{BillingLine, compute_totals},
		invoice::{Invoice, InvoiceLine, InvoiceType},
	},
};

/// Format a `DateTime<Utc>` as `dd/mm/yyyy`.
fn fmt_date(dt: &DateTime<Utc>) -> String {
	format!("{:02}/{:02}/{}", dt.format("%d"), dt.format("%m"), dt.format("%Y"))
}

/// Build a `pdf::DocumentSnapshot` from the domain entities gathered for an invoice.
///
/// `emitter` is the selected `OrganizationContext` that acts as the seller on
/// the document. Its identity fields (`label`, `siret`, `iban`, …) are mapped
/// directly to `SellerInfo`; `settings` is used only for `payment_terms` and
/// `footer`.
///
/// `mention_bodies` are the resolved template bodies in the order they appear in
/// `invoice.legal_mention_template_ids`.
pub fn build_invoice_snapshot(
	invoice: &Invoice,
	emitter: &OrganizationContext,
	settings: Option<&BillingSettings>,
	customer: &Customer,
	context: Option<&CustomerContext>,
	mention_bodies: &[String],
) -> pdf::DocumentSnapshot {
	// ── Kind & title ─────────────────────────────────────────────────────────
	let kind = match invoice.invoice_type {
		InvoiceType::Standard => "INVOICE",
		InvoiceType::Deposit => "DEPOSIT",
		InvoiceType::Balance => "BALANCE",
	}
	.to_owned();

	let title = invoice.title.clone();
	let reference = invoice.reference.clone();

	// ── Dates ────────────────────────────────────────────────────────────────
	let issued_date = invoice.issued_at.as_ref().map(fmt_date);
	let due_date = invoice.due_at.as_ref().map(fmt_date);

	// ── Seller ───────────────────────────────────────────────────────────────
	let seller_address = {
		let parts: Vec<String> = [
			emitter.address_line.clone(),
			emitter.postal_code
				.as_ref()
				.and_then(|pc| emitter.city.as_ref().map(|c| format!("{pc} {c}")))
				.or_else(|| emitter.postal_code.clone())
				.or_else(|| emitter.city.clone()),
			emitter.country.clone(),
		]
		.into_iter()
		.flatten()
		.filter(|s| !s.is_empty())
		.collect();
		if parts.is_empty() { None } else { Some(parts.join(", ")) }
	};
	let seller = pdf::SellerInfo {
		name: emitter.label.clone(),
		address: seller_address,
		siret: emitter.siret.clone(),
		rcs: emitter.rcs.clone(),
		ape: emitter.ape.clone(),
		vat_intracom: emitter.vat_intracom.clone(),
		iban: emitter.iban.clone(),
		bic: emitter.bic.clone(),
	};

	// ── Customer ─────────────────────────────────────────────────────────────
	let customer_name = format!("{} {}", customer.first_name.trim(), customer.last_name.trim());
	let customer_address = context.and_then(|ctx| {
		// Build a single-string address from the available fields.
		let parts: Vec<String> = [
			ctx.address_line.clone(),
			ctx.postal_code
				.as_ref()
				.and_then(|pc| ctx.city.as_ref().map(|c| format!("{pc} {c}")))
				.or_else(|| ctx.postal_code.clone())
				.or_else(|| ctx.city.clone()),
		]
		.into_iter()
		.flatten()
		.filter(|s| !s.is_empty())
		.collect();
		if parts.is_empty() { None } else { Some(parts.join(", ")) }
	});

	let customer_info = pdf::CustomerInfo {
		name: customer_name,
		address: customer_address,
	};

	// ── Lines ────────────────────────────────────────────────────────────────
	let active_lines: Vec<&InvoiceLine> =
		invoice.lines.iter().filter(|l| l.deleted_at.is_none()).collect();

	let lines: Vec<pdf::LineSnapshot> = active_lines
		.iter()
		.map(|l| {
			let line_ht_cents =
				round_line_ht(l.quantity, l.unit_price_cents as i64);
			pdf::LineSnapshot {
				label: l.label.clone(),
				quantity: l.quantity.to_string(),
				unit: l.unit.as_str().to_owned(),
				unit_price_cents: l.unit_price_cents as i64,
				vat_rate: l.vat_rate.to_string(),
				line_ht_cents,
			}
		})
		.collect();

	// ── VAT breakdown ────────────────────────────────────────────────────────
	let billing_lines: Vec<BillingLine> = active_lines
		.iter()
		.map(|l| BillingLine {
			quantity: l.quantity,
			unit_price_cents: l.unit_price_cents as i64,
			vat_rate: l.vat_rate,
		})
		.collect();
	let totals = compute_totals(&billing_lines);

	let vat_breakdown: Vec<pdf::VatBucketSnapshot> = totals
		.vat_breakdown
		.iter()
		.map(|b| pdf::VatBucketSnapshot {
			rate: b.rate.to_string(),
			base_ht_cents: b.base_ht_cents,
			vat_cents: b.vat_cents,
		})
		.collect();

	// ── Totals (from the already-computed invoice fields, cast to i64) ───────
	let total_ht_cents = invoice.total_ht_cents as i64;
	let total_vat_cents = invoice.total_vat_cents as i64;
	let total_ttc_cents = invoice.total_ttc_cents as i64;

	// ── Deposit / balance fields ─────────────────────────────────────────────
	let (deposit_label, deposit_amount_cents, remaining_cents) =
		match invoice.invoice_type {
			InvoiceType::Deposit => (
				Some("Acompte".to_owned()),
				invoice.deposit_amount_cents.map(|v| v as i64),
				None,
			),
			InvoiceType::Balance => (
				Some("Acompte déduit".to_owned()),
				invoice.deposit_amount_cents.map(|v| v as i64),
				Some(invoice.total_ttc_cents as i64),
			),
			InvoiceType::Standard => (None, None, None),
		};

	// ── Footer / payment terms ────────────────────────────────────────────────
	let footer = settings.and_then(|s| s.footer.clone());
	let payment_terms = settings.map(|s| {
		format!("Paiement sous {} jours", s.payment_terms_days)
	});

	pdf::DocumentSnapshot {
		kind,
		reference,
		title,
		issued_date,
		due_date,
		seller,
		customer: customer_info,
		lines,
		vat_breakdown,
		total_ht_cents,
		total_vat_cents,
		total_ttc_cents,
		deposit_label,
		deposit_amount_cents,
		remaining_cents,
		legal_mentions: mention_bodies.to_vec(),
		footer,
		payment_terms,
	}
}

/// `round(quantity * unit_price_cents)` using half-up, matching the billing module.
fn round_line_ht(quantity: Decimal, unit_price_cents: i64) -> i64 {
	use rust_decimal::RoundingStrategy;
	let raw = quantity * Decimal::from(unit_price_cents);
	raw.round_dp_with_strategy(0, RoundingStrategy::MidpointAwayFromZero)
		.try_into()
		.expect("line HT out of i64 range")
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::{
		CustomerContextId, CustomerId, InvoiceId, InvoiceLineId, InvoiceStatus, InvoiceType,
		OrganizationContextId, OrganizationId, ServiceRateUnit,
	};
	use chrono::Utc;
	use rust_decimal::Decimal;
	use uuid::Uuid;

	fn make_invoice(invoice_type: InvoiceType, with_issued: bool) -> Invoice {
		let now = Utc::now();
		let org_id = OrganizationId(Uuid::new_v4());
		let invoice_id = InvoiceId(Uuid::new_v4());
		Invoice {
			id: invoice_id,
			org_id,
			customer_id: CustomerId(Uuid::new_v4()),
			customer_context_id: CustomerContextId(Uuid::new_v4()),
			emitter_context_id: None,
			reference: Some("FAC-2026-0001".to_owned()),
			title: "Facture cuisine".to_owned(),
			status: InvoiceStatus::Sent,
			invoice_type,
			source_quote_id: None,
			parent_invoice_id: None,
			deposit_basis: None,
			deposit_value: None,
			deposit_amount_cents: if invoice_type == InvoiceType::Balance {
				Some(36_000)
			} else {
				None
			},
			total_ht_cents: 10_000,
			total_vat_cents: 2_000,
			total_ttc_cents: 12_000,
			total_cents: 12_000,
			amount_paid_cents: 0,
			pdf_key: None,
			issued_at: if with_issued { Some(now) } else { None },
			due_at: None,
			lines: vec![crate::domain::invoice::InvoiceLine {
				id: InvoiceLineId(Uuid::new_v4()),
				org_id,
				invoice_id,
				service_rate_id: None,
				label: "Pose carrelage".to_owned(),
				quantity: Decimal::from(1u32),
				unit: ServiceRateUnit::M2,
				unit_price_cents: 10_000,
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

	fn make_customer() -> Customer {
		let now = Utc::now();
		Customer {
			id: CustomerId(Uuid::new_v4()),
			organization_id: OrganizationId(Uuid::new_v4()),
			status: crate::domain::customer::CustomerStatus::Client,
			pipeline_stage: crate::domain::customer::CustomerPipelineStage::Won,
			last_name: "Dupont".to_owned(),
			first_name: "Jean".to_owned(),
			phone: None,
			email: None,
			deleted_at: None,
			created_at: now,
			updated_at: now,
		}
	}

	fn make_emitter(org_id: OrganizationId) -> OrganizationContext {
		let now = Utc::now();
		OrganizationContext {
			id: OrganizationContextId(Uuid::new_v4()),
			org_id,
			label: "Mon Atelier SAS".to_owned(),
			address_line: Some("12 rue des Artisans".to_owned()),
			postal_code: Some("75011".to_owned()),
			city: Some("Paris".to_owned()),
			country: Some("France".to_owned()),
			siret: Some("12345678901234".to_owned()),
			rcs: None,
			ape: Some("4321A".to_owned()),
			vat_intracom: Some("FR12123456789".to_owned()),
			iban: Some("FR76123".to_owned()),
			bic: Some("BNPAFRPP".to_owned()),
			deleted_at: None,
			created_at: now,
			updated_at: now,
		}
	}

	#[test]
	fn snapshot_standard_invoice_maps_fields_correctly() {
		let invoice = make_invoice(InvoiceType::Standard, true);
		let customer = make_customer();
		let emitter = make_emitter(invoice.org_id);
		let snapshot = build_invoice_snapshot(
			&invoice,
			&emitter,
			None,
			&customer,
			None,
			&["Mention légale A".to_owned()],
		);

		assert_eq!(snapshot.kind, "INVOICE");
		assert_eq!(snapshot.reference.as_deref(), Some("FAC-2026-0001"));
		assert_eq!(snapshot.title, "Facture cuisine");
		assert!(snapshot.issued_date.is_some());
		// seller comes from the emitter context
		assert_eq!(snapshot.seller.name, "Mon Atelier SAS");
		assert_eq!(snapshot.seller.siret.as_deref(), Some("12345678901234"));
		assert_eq!(snapshot.seller.iban.as_deref(), Some("FR76123"));
		let seller_addr = snapshot.seller.address.as_deref().unwrap();
		assert!(seller_addr.contains("12 rue des Artisans"), "seller_addr={seller_addr}");
		assert!(seller_addr.contains("75011"), "seller_addr={seller_addr}");
		assert!(seller_addr.contains("Paris"), "seller_addr={seller_addr}");
		assert!(seller_addr.contains("France"), "seller_addr={seller_addr}");
		assert_eq!(snapshot.customer.name, "Jean Dupont");
		assert_eq!(snapshot.lines.len(), 1);
		assert_eq!(snapshot.lines[0].label, "Pose carrelage");
		assert_eq!(snapshot.lines[0].line_ht_cents, 10_000);
		assert_eq!(snapshot.total_ht_cents, 10_000);
		assert_eq!(snapshot.total_vat_cents, 2_000);
		assert_eq!(snapshot.total_ttc_cents, 12_000);
		assert_eq!(snapshot.legal_mentions, vec!["Mention légale A".to_owned()]);
		assert_eq!(snapshot.deposit_label, None);
		assert_eq!(snapshot.deposit_amount_cents, None);
		assert_eq!(snapshot.remaining_cents, None);
	}

	#[test]
	fn snapshot_balance_invoice_sets_remaining_and_deposit_fields() {
		let mut invoice = make_invoice(InvoiceType::Balance, true);
		invoice.deposit_amount_cents = Some(36_000);
		let customer = make_customer();
		let emitter = make_emitter(invoice.org_id);
		let snapshot = build_invoice_snapshot(
			&invoice,
			&emitter,
			None,
			&customer,
			None,
			&[],
		);

		assert_eq!(snapshot.kind, "BALANCE");
		assert_eq!(snapshot.deposit_label.as_deref(), Some("Acompte déduit"));
		assert_eq!(snapshot.deposit_amount_cents, Some(36_000));
		assert_eq!(snapshot.remaining_cents, Some(12_000));
	}

	#[test]
	fn snapshot_with_billing_settings_populates_payment_terms_and_footer() {
		use chrono::Utc;
		use rust_decimal::Decimal;
		let invoice = make_invoice(InvoiceType::Standard, false);
		let customer = make_customer();
		let emitter = make_emitter(invoice.org_id);
		let settings = BillingSettings {
			org_id: invoice.org_id,
			payment_terms_days: 30,
			late_penalty_rate: Decimal::from(5u32),
			recovery_indemnity_cents: 4000,
			default_deposit_basis: None,
			default_deposit_value: None,
			default_vat_rate: Decimal::from(20u32),
			iban: Some("FR76999".to_owned()),
			bic: Some("BNPAFRPP".to_owned()),
			siret: Some("99999999999999".to_owned()),
			rcs: None,
			ape: Some("0000Z".to_owned()),
			vat_intracom: Some("FR99999999999".to_owned()),
			footer: Some("Pied de page légal".to_owned()),
			created_at: Utc::now(),
			updated_at: Utc::now(),
		};

		let snapshot = build_invoice_snapshot(
			&invoice,
			&emitter,
			Some(&settings),
			&customer,
			None,
			&[],
		);

		// seller identity still comes from the emitter context, NOT from billing settings
		assert_eq!(snapshot.seller.name, "Mon Atelier SAS");
		assert_eq!(snapshot.seller.siret.as_deref(), Some("12345678901234"));
		assert_eq!(snapshot.seller.iban.as_deref(), Some("FR76123"));
		assert_eq!(snapshot.seller.vat_intracom.as_deref(), Some("FR12123456789"));
		// payment terms and footer come from billing settings
		assert_eq!(snapshot.footer.as_deref(), Some("Pied de page légal"));
		assert!(snapshot.payment_terms.as_ref().unwrap().contains("30"));
	}

	#[test]
	fn snapshot_with_customer_context_builds_address() {
		let invoice = make_invoice(InvoiceType::Standard, false);
		let customer = make_customer();
		let emitter = make_emitter(invoice.org_id);
		let now = Utc::now();
		let context = CustomerContext {
			id: CustomerContextId(Uuid::new_v4()),
			customer_id: customer.id,
			label: "Chantier cuisine".to_owned(),
			address_line: Some("12 rue de la Paix".to_owned()),
			postal_code: Some("75001".to_owned()),
			city: Some("Paris".to_owned()),
			photo_key: None,
			deleted_at: None,
			created_at: now,
			updated_at: now,
		};

		let snapshot = build_invoice_snapshot(
			&invoice,
			&emitter,
			None,
			&customer,
			Some(&context),
			&[],
		);

		let addr = snapshot.customer.address.unwrap();
		assert!(addr.contains("12 rue de la Paix"), "addr={addr}");
		assert!(addr.contains("75001"), "addr={addr}");
		assert!(addr.contains("Paris"), "addr={addr}");
	}

	#[test]
	fn snapshot_seller_address_is_none_when_emitter_has_no_address_fields() {
		let now = Utc::now();
		let org_id = OrganizationId(Uuid::new_v4());
		let emitter = OrganizationContext {
			id: OrganizationContextId(Uuid::new_v4()),
			org_id,
			label: "Atelier Sans Adresse".to_owned(),
			address_line: None,
			postal_code: None,
			city: None,
			country: None,
			siret: None,
			rcs: None,
			ape: None,
			vat_intracom: None,
			iban: None,
			bic: None,
			deleted_at: None,
			created_at: now,
			updated_at: now,
		};
		let invoice = make_invoice(InvoiceType::Standard, false);
		let customer = make_customer();
		let snapshot = build_invoice_snapshot(&invoice, &emitter, None, &customer, None, &[]);

		assert_eq!(snapshot.seller.name, "Atelier Sans Adresse");
		assert!(snapshot.seller.address.is_none());
	}
}
