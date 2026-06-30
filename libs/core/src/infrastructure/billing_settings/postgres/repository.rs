use common::CoreError;
use mestier_macros::repository;

use crate::{
	BillingSettings, OrganizationId,
	domain::billing_settings::{
		commands::UpsertBillingSettingsCommand,
		ports::BillingSettingsRepository,
	},
	infrastructure::{
		billing_settings::postgres::model::BillingSettingsRow,
		postgres::{SharedTx, error::map_sqlx_error},
	},
};

#[repository(domain = BillingSettings, backend = Postgres)]
pub struct PgBillingSettingsRepository<'tx> {
	tx: SharedTx<'tx>,
}

impl<'tx> PgBillingSettingsRepository<'tx> {
	pub fn new(tx: &SharedTx<'tx>) -> Self {
		Self { tx: tx.clone() }
	}
}

impl<'tx> BillingSettingsRepository for PgBillingSettingsRepository<'tx> {
	async fn find_by_org(
		&mut self,
		org_id: OrganizationId,
	) -> Result<Option<BillingSettings>, CoreError> {
		let mut tx = self.tx.lock().await;
		let row = sqlx::query_as!(
			BillingSettingsRow,
			r#"
            SELECT
                org_id, payment_terms_days, late_penalty_rate, recovery_indemnity_cents,
                default_deposit_basis, default_deposit_value, default_vat_rate,
                iban, bic, siret, rcs, ape, vat_intracom, footer,
                created_at, updated_at
            FROM billing_settings
            WHERE org_id = $1
            "#,
			org_id.0,
		)
		.fetch_optional(&mut ***tx)
		.await
		.map_err(map_sqlx_error)?;

		row.map(TryInto::try_into).transpose()
	}

	async fn upsert(
		&mut self,
		cmd: &UpsertBillingSettingsCommand,
	) -> Result<BillingSettings, CoreError> {
		let mut tx = self.tx.lock().await;
		let row = sqlx::query_as!(
			BillingSettingsRow,
			r#"
            INSERT INTO billing_settings (
                org_id, payment_terms_days, late_penalty_rate, recovery_indemnity_cents,
                default_deposit_basis, default_deposit_value, default_vat_rate,
                iban, bic, siret, rcs, ape, vat_intracom, footer,
                created_at, updated_at
            ) VALUES (
                $1, $2, $3, $4, $5, $6, $7,
                $8, $9, $10, $11, $12, $13, $14,
                now(), now()
            )
            ON CONFLICT (org_id) DO UPDATE SET
                payment_terms_days      = EXCLUDED.payment_terms_days,
                late_penalty_rate       = EXCLUDED.late_penalty_rate,
                recovery_indemnity_cents = EXCLUDED.recovery_indemnity_cents,
                default_deposit_basis   = EXCLUDED.default_deposit_basis,
                default_deposit_value   = EXCLUDED.default_deposit_value,
                default_vat_rate        = EXCLUDED.default_vat_rate,
                iban                    = EXCLUDED.iban,
                bic                     = EXCLUDED.bic,
                siret                   = EXCLUDED.siret,
                rcs                     = EXCLUDED.rcs,
                ape                     = EXCLUDED.ape,
                vat_intracom            = EXCLUDED.vat_intracom,
                footer                  = EXCLUDED.footer,
                updated_at              = now()
            RETURNING
                org_id, payment_terms_days, late_penalty_rate, recovery_indemnity_cents,
                default_deposit_basis, default_deposit_value, default_vat_rate,
                iban, bic, siret, rcs, ape, vat_intracom, footer,
                created_at, updated_at
            "#,
			cmd.org_id.0,
			cmd.payment_terms_days,
			cmd.late_penalty_rate,
			cmd.recovery_indemnity_cents,
			cmd.default_deposit_basis,
			cmd.default_deposit_value,
			cmd.default_vat_rate,
			cmd.iban,
			cmd.bic,
			cmd.siret,
			cmd.rcs,
			cmd.ape,
			cmd.vat_intracom,
			cmd.footer,
		)
		.fetch_one(&mut ***tx)
		.await
		.map_err(map_sqlx_error)?;

		row.try_into()
	}
}
