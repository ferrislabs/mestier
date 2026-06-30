use rust_decimal::Decimal;

use crate::OrganizationId;

#[derive(Debug, Clone)]
pub struct UpsertBillingSettingsCommand {
	pub org_id: OrganizationId,
	pub payment_terms_days: i32,
	pub late_penalty_rate: Decimal,
	pub recovery_indemnity_cents: i32,
	pub default_deposit_basis: Option<String>,
	pub default_deposit_value: Option<Decimal>,
	pub default_vat_rate: Decimal,
	pub iban: Option<String>,
	pub bic: Option<String>,
	pub siret: Option<String>,
	pub rcs: Option<String>,
	pub ape: Option<String>,
	pub vat_intracom: Option<String>,
	pub footer: Option<String>,
}
