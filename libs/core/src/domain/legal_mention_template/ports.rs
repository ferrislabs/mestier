use chrono::{DateTime, Utc};
use common::CoreError;

use crate::{LegalMentionTemplate, LegalMentionTemplateId, OrganizationId};

#[cfg_attr(test, mockall::automock)]
pub trait LegalMentionTemplateRepository: Send {
	fn insert(
		&mut self,
		template: &LegalMentionTemplate,
	) -> impl Future<Output = Result<LegalMentionTemplate, CoreError>> + Send;

	fn find_by_id(
		&mut self,
		id: LegalMentionTemplateId,
	) -> impl Future<Output = Result<Option<LegalMentionTemplate>, CoreError>> + Send;

	fn list_by_organization(
		&mut self,
		org_id: OrganizationId,
		limit: u64,
		offset: u64,
	) -> impl Future<Output = Result<(Vec<LegalMentionTemplate>, u64), CoreError>> + Send;

	fn update(
		&mut self,
		template: &LegalMentionTemplate,
	) -> impl Future<Output = Result<LegalMentionTemplate, CoreError>> + Send;

	fn soft_delete(
		&mut self,
		id: LegalMentionTemplateId,
		deleted_at: DateTime<Utc>,
	) -> impl Future<Output = Result<(), CoreError>> + Send;
}
