use chrono::{DateTime, Utc};
use common::CoreError;

use crate::{OrganizationContext, OrganizationContextId, OrganizationId};

#[cfg_attr(test, mockall::automock)]
pub trait OrganizationContextRepository: Send {
	fn insert(
		&mut self,
		organization_context: &OrganizationContext,
	) -> impl Future<Output = Result<OrganizationContext, CoreError>> + Send;

	fn find_by_id(
		&mut self,
		id: OrganizationContextId,
	) -> impl Future<Output = Result<Option<OrganizationContext>, CoreError>> + Send;

	fn list_by_organization(
		&mut self,
		org_id: OrganizationId,
		limit: u64,
		offset: u64,
	) -> impl Future<Output = Result<(Vec<OrganizationContext>, u64), CoreError>> + Send;

	fn update(
		&mut self,
		organization_context: &OrganizationContext,
	) -> impl Future<Output = Result<OrganizationContext, CoreError>> + Send;

	fn soft_delete(
		&mut self,
		id: OrganizationContextId,
		deleted_at: DateTime<Utc>,
	) -> impl Future<Output = Result<(), CoreError>> + Send;
}
