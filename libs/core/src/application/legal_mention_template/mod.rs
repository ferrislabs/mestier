use common::CoreError;
use mestier_macros::transactional;

use crate::{
	LegalMentionTemplate, LegalMentionTemplateId, OrganizationId,
	application::MestierUseCase,
	domain::legal_mention_template::{
		commands::{CreateLegalMentionTemplateCommand, UpdateLegalMentionTemplateCommand},
		service::LegalMentionTemplateService,
	},
};

impl MestierUseCase {
	#[transactional(legal_mention_template)]
	pub async fn create_legal_mention_template(
		&self,
		command: CreateLegalMentionTemplateCommand,
	) -> Result<LegalMentionTemplate, CoreError> {
		let mut service = LegalMentionTemplateService::new(legal_mention_template_repository);
		service.create_legal_mention_template(command).await
	}

	#[transactional(legal_mention_template)]
	pub async fn get_legal_mention_template(
		&self,
		id: LegalMentionTemplateId,
	) -> Result<LegalMentionTemplate, CoreError> {
		let mut service = LegalMentionTemplateService::new(legal_mention_template_repository);
		service.get_legal_mention_template(id).await
	}

	#[transactional(legal_mention_template)]
	pub async fn list_legal_mention_templates(
		&self,
		org_id: OrganizationId,
		limit: u64,
		offset: u64,
	) -> Result<(Vec<LegalMentionTemplate>, u64), CoreError> {
		let mut service = LegalMentionTemplateService::new(legal_mention_template_repository);
		service
			.list_legal_mention_templates(org_id, limit, offset)
			.await
	}

	#[transactional(legal_mention_template)]
	pub async fn update_legal_mention_template(
		&self,
		command: UpdateLegalMentionTemplateCommand,
	) -> Result<LegalMentionTemplate, CoreError> {
		let mut service = LegalMentionTemplateService::new(legal_mention_template_repository);
		service.update_legal_mention_template(command).await
	}

	#[transactional(legal_mention_template)]
	pub async fn soft_delete_legal_mention_template(
		&self,
		id: LegalMentionTemplateId,
	) -> Result<(), CoreError> {
		let mut service = LegalMentionTemplateService::new(legal_mention_template_repository);
		service.soft_delete_legal_mention_template(id).await
	}
}
