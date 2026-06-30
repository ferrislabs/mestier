use common::CoreError;

use crate::{
	BillingSettings, OrganizationId,
	domain::billing_settings::{
		commands::UpsertBillingSettingsCommand,
		ports::BillingSettingsRepository,
	},
};

pub struct BillingSettingsService<R>
where
	R: BillingSettingsRepository,
{
	repo: R,
}

impl<R> BillingSettingsService<R>
where
	R: BillingSettingsRepository,
{
	pub fn new(repo: R) -> Self {
		Self { repo }
	}

	pub async fn get_billing_settings(
		&mut self,
		org_id: OrganizationId,
	) -> Result<Option<BillingSettings>, CoreError> {
		self.repo.find_by_org(org_id).await
	}

	pub async fn upsert_billing_settings(
		&mut self,
		command: UpsertBillingSettingsCommand,
	) -> Result<BillingSettings, CoreError> {
		self.repo.upsert(&command).await
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::{
		OrganizationId,
		domain::billing_settings::{
			commands::UpsertBillingSettingsCommand,
			ports::MockBillingSettingsRepository,
		},
	};
	use chrono::Utc;
	use rust_decimal::Decimal;
	use uuid::Uuid;

	fn org_id() -> OrganizationId {
		OrganizationId(Uuid::new_v4())
	}

	fn sample_settings(org_id: OrganizationId) -> BillingSettings {
		let now = Utc::now();
		BillingSettings {
			org_id,
			payment_terms_days: 30,
			late_penalty_rate: Decimal::ZERO,
			recovery_indemnity_cents: 4000,
			default_deposit_basis: None,
			default_deposit_value: None,
			default_vat_rate: Decimal::from(20u32),
			iban: None,
			bic: None,
			siret: None,
			rcs: None,
			ape: None,
			vat_intracom: None,
			footer: None,
			created_at: now,
			updated_at: now,
		}
	}

	fn upsert_cmd(org_id: OrganizationId) -> UpsertBillingSettingsCommand {
		UpsertBillingSettingsCommand {
			org_id,
			payment_terms_days: 30,
			late_penalty_rate: Decimal::ZERO,
			recovery_indemnity_cents: 4000,
			default_deposit_basis: None,
			default_deposit_value: None,
			default_vat_rate: Decimal::from(20u32),
			iban: None,
			bic: None,
			siret: None,
			rcs: None,
			ape: None,
			vat_intracom: None,
			footer: None,
		}
	}

	#[tokio::test]
	async fn get_returns_none_when_no_settings_exist() {
		let oid = org_id();
		let mut repo = MockBillingSettingsRepository::new();
		repo.expect_find_by_org()
			.returning(|_| Box::pin(async { Ok(None) }));

		let mut service = BillingSettingsService::new(repo);
		let result = service.get_billing_settings(oid).await.unwrap();
		assert!(result.is_none());
	}

	#[tokio::test]
	async fn upsert_is_idempotent() {
		let oid = org_id();
		let settings = sample_settings(oid);
		let settings_clone = settings.clone();

		let mut repo = MockBillingSettingsRepository::new();
		// Called twice — must return the same value each time.
		repo.expect_upsert().times(2).returning(move |_| {
			let s = settings_clone.clone();
			Box::pin(async move { Ok(s) })
		});

		let mut service = BillingSettingsService::new(repo);

		let first = service
			.upsert_billing_settings(upsert_cmd(oid))
			.await
			.unwrap();
		let second = service
			.upsert_billing_settings(upsert_cmd(oid))
			.await
			.unwrap();

		assert_eq!(first, second);
		assert_eq!(first.org_id, settings.org_id);
	}
}
