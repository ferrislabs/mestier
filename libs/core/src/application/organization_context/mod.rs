use common::CoreError;
use mestier_macros::transactional;

use crate::{
	OrganizationContext, OrganizationContextId, OrganizationId,
	application::MestierUseCase,
	domain::organization_context::{
		commands::{CreateOrganizationContextCommand, UpdateOrganizationContextCommand},
		service::OrganizationContextService,
	},
};

impl MestierUseCase {
	#[transactional(organization_context)]
	pub async fn create_organization_context(
		&self,
		command: CreateOrganizationContextCommand,
	) -> Result<OrganizationContext, CoreError> {
		let mut service = OrganizationContextService::new(organization_context_repository);
		service.create_organization_context(command).await
	}

	#[transactional(organization_context)]
	pub async fn get_organization_context(
		&self,
		id: OrganizationContextId,
	) -> Result<OrganizationContext, CoreError> {
		let mut service = OrganizationContextService::new(organization_context_repository);
		service.get_organization_context(id).await
	}

	#[transactional(organization_context)]
	pub async fn list_organization_contexts(
		&self,
		org_id: OrganizationId,
		limit: u64,
		offset: u64,
	) -> Result<(Vec<OrganizationContext>, u64), CoreError> {
		let mut service = OrganizationContextService::new(organization_context_repository);
		service
			.list_organization_contexts(org_id, limit, offset)
			.await
	}

	#[transactional(organization_context)]
	pub async fn update_organization_context(
		&self,
		command: UpdateOrganizationContextCommand,
	) -> Result<OrganizationContext, CoreError> {
		let mut service = OrganizationContextService::new(organization_context_repository);
		service.update_organization_context(command).await
	}

	#[transactional(organization_context)]
	pub async fn soft_delete_organization_context(
		&self,
		id: OrganizationContextId,
	) -> Result<(), CoreError> {
		let mut service = OrganizationContextService::new(organization_context_repository);
		service.soft_delete_organization_context(id).await
	}
}
