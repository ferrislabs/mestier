use chrono::{DateTime, Utc};
use common::CoreError;

use crate::{LegalMentionTemplateId, OrganizationId};
use crate::domain::invoice::{Invoice, InvoiceId, InvoiceStatus};

#[cfg_attr(test, mockall::automock)]
pub trait InvoiceRepository: Send {
	fn next_reference(
		&mut self,
		org_id: OrganizationId,
		year: i32,
	) -> impl Future<Output = Result<String, CoreError>> + Send;

	fn insert(
		&mut self,
		invoice: &Invoice,
	) -> impl Future<Output = Result<Invoice, CoreError>> + Send;

	fn find_by_id(
		&mut self,
		id: InvoiceId,
	) -> impl Future<Output = Result<Option<Invoice>, CoreError>> + Send;

	fn list_by_organization(
		&mut self,
		org_id: OrganizationId,
		limit: u64,
		offset: u64,
	) -> impl Future<Output = Result<(Vec<Invoice>, u64), CoreError>> + Send;

	fn update(
		&mut self,
		invoice: &Invoice,
	) -> impl Future<Output = Result<Invoice, CoreError>> + Send;

	fn update_status(
		&mut self,
		id: InvoiceId,
		status: InvoiceStatus,
		reference: Option<String>,
		issued_at: Option<DateTime<Utc>>,
		updated_at: DateTime<Utc>,
	) -> impl Future<Output = Result<Invoice, CoreError>> + Send;

	fn soft_delete(
		&mut self,
		id: InvoiceId,
		deleted_at: DateTime<Utc>,
	) -> impl Future<Output = Result<(), CoreError>> + Send;

	fn replace_legal_mention_templates(
		&mut self,
		invoice: &Invoice,
		template_ids: &[LegalMentionTemplateId],
	) -> impl Future<Output = Result<(), CoreError>> + Send;

	fn find_legal_mention_template_ids(
		&mut self,
		invoice_id: InvoiceId,
	) -> impl Future<Output = Result<Vec<LegalMentionTemplateId>, CoreError>> + Send;

	fn list_deposits_for_parent(
		&mut self,
		parent_id: InvoiceId,
	) -> impl Future<Output = Result<Vec<Invoice>, CoreError>> + Send;
}
