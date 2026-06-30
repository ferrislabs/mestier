use chrono::{DateTime, Utc};
use common::CoreError;
use rust_decimal::Decimal;
use uuid::Uuid;

use crate::{BillingSettings, OrganizationId};

#[derive(Debug, Clone)]
pub struct BillingSettingsRow {
	pub org_id: Uuid,
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
	pub created_at: DateTime<Utc>,
	pub updated_at: DateTime<Utc>,
}

impl TryFrom<BillingSettingsRow> for BillingSettings {
	type Error = CoreError;

	fn try_from(row: BillingSettingsRow) -> Result<Self, Self::Error> {
		Ok(Self {
			org_id: OrganizationId(row.org_id),
			payment_terms_days: row.payment_terms_days,
			late_penalty_rate: row.late_penalty_rate,
			recovery_indemnity_cents: row.recovery_indemnity_cents,
			default_deposit_basis: row.default_deposit_basis,
			default_deposit_value: row.default_deposit_value,
			default_vat_rate: row.default_vat_rate,
			iban: row.iban,
			bic: row.bic,
			siret: row.siret,
			rcs: row.rcs,
			ape: row.ape,
			vat_intracom: row.vat_intracom,
			footer: row.footer,
			created_at: row.created_at,
			updated_at: row.updated_at,
		})
	}
}
