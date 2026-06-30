use common::CoreError;
use mestier_macros::transactional;

use crate::{
	BillingSettings, OrganizationId,
	application::MestierUseCase,
	domain::billing_settings::{
		commands::UpsertBillingSettingsCommand,
		service::BillingSettingsService,
	},
};

impl MestierUseCase {
	#[transactional(billing_settings)]
	pub async fn get_billing_settings(
		&self,
		org_id: OrganizationId,
	) -> Result<Option<BillingSettings>, CoreError> {
		let mut service = BillingSettingsService::new(billing_settings_repository);
		service.get_billing_settings(org_id).await
	}

	#[transactional(billing_settings)]
	pub async fn upsert_billing_settings(
		&self,
		command: UpsertBillingSettingsCommand,
	) -> Result<BillingSettings, CoreError> {
		let mut service = BillingSettingsService::new(billing_settings_repository);
		service.upsert_billing_settings(command).await
	}
}
