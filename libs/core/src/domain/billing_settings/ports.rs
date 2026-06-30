use common::CoreError;

use crate::{
	BillingSettings, OrganizationId,
	domain::billing_settings::commands::UpsertBillingSettingsCommand,
};

#[cfg_attr(test, mockall::automock)]
pub trait BillingSettingsRepository: Send {
	fn find_by_org(
		&mut self,
		org_id: OrganizationId,
	) -> impl Future<Output = Result<Option<BillingSettings>, CoreError>> + Send;

	fn upsert(
		&mut self,
		command: &UpsertBillingSettingsCommand,
	) -> impl Future<Output = Result<BillingSettings, CoreError>> + Send;
}
