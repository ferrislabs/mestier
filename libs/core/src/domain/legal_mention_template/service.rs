use chrono::Utc;
use common::{CoreError, generate_uuid_v7};

use crate::{
	LegalMentionTemplate, LegalMentionTemplateId, OrganizationId,
	domain::legal_mention_template::{
		commands::{CreateLegalMentionTemplateCommand, UpdateLegalMentionTemplateCommand},
		ports::LegalMentionTemplateRepository,
	},
};

pub struct LegalMentionTemplateService<R>
where
	R: LegalMentionTemplateRepository,
{
	repo: R,
}

impl<R> LegalMentionTemplateService<R>
where
	R: LegalMentionTemplateRepository,
{
	pub fn new(repo: R) -> Self {
		Self { repo }
	}

	pub async fn create_legal_mention_template(
		&mut self,
		command: CreateLegalMentionTemplateCommand,
	) -> Result<LegalMentionTemplate, CoreError> {
		validate_name(&command.name)?;
		validate_body(&command.body)?;

		let now = Utc::now();
		self.repo
			.insert(&LegalMentionTemplate {
				id: LegalMentionTemplateId(generate_uuid_v7()),
				org_id: command.org_id,
				name: command.name,
				body: command.body,
				created_at: now,
				updated_at: now,
				deleted_at: None,
			})
			.await
	}

	pub async fn get_legal_mention_template(
		&mut self,
		id: LegalMentionTemplateId,
	) -> Result<LegalMentionTemplate, CoreError> {
		self.repo.find_by_id(id).await?.ok_or(CoreError::NotFound)
	}

	pub async fn list_legal_mention_templates(
		&mut self,
		org_id: OrganizationId,
		limit: u64,
		offset: u64,
	) -> Result<(Vec<LegalMentionTemplate>, u64), CoreError> {
		self.repo.list_by_organization(org_id, limit, offset).await
	}

	pub async fn update_legal_mention_template(
		&mut self,
		command: UpdateLegalMentionTemplateCommand,
	) -> Result<LegalMentionTemplate, CoreError> {
		validate_name(&command.name)?;
		validate_body(&command.body)?;

		let mut template = self.get_legal_mention_template(command.id).await?;
		template.name = command.name;
		template.body = command.body;
		template.updated_at = Utc::now();

		self.repo.update(&template).await
	}

	pub async fn soft_delete_legal_mention_template(
		&mut self,
		id: LegalMentionTemplateId,
	) -> Result<(), CoreError> {
		self.get_legal_mention_template(id).await?;
		self.repo.soft_delete(id, Utc::now()).await
	}
}

fn validate_name(name: &str) -> Result<(), CoreError> {
	if name.trim().is_empty() {
		return Err(CoreError::Conflict(
			"legal mention template name cannot be empty".to_owned(),
		));
	}
	Ok(())
}

fn validate_body(body: &str) -> Result<(), CoreError> {
	if body.trim().is_empty() {
		return Err(CoreError::Conflict(
			"legal mention template body cannot be empty".to_owned(),
		));
	}
	Ok(())
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::{
		LegalMentionTemplateId, OrganizationId,
		domain::legal_mention_template::{
			commands::CreateLegalMentionTemplateCommand,
			ports::MockLegalMentionTemplateRepository,
		},
	};
	use chrono::Utc;
	use mockall::predicate::eq;
	use uuid::Uuid;

	fn template(id: LegalMentionTemplateId) -> LegalMentionTemplate {
		let now = Utc::now();
		LegalMentionTemplate {
			id,
			org_id: OrganizationId(Uuid::new_v4()),
			name: "Mentions légales standard".to_owned(),
			body: "Conformément à la loi…".to_owned(),
			created_at: now,
			updated_at: now,
			deleted_at: None,
		}
	}

	#[tokio::test]
	async fn create_inserts_template_with_generated_id() {
		let mut repo = MockLegalMentionTemplateRepository::new();
		repo.expect_insert().times(1).returning(|t| {
			let t = t.clone();
			Box::pin(async move { Ok(t) })
		});

		let mut service = LegalMentionTemplateService::new(repo);
		let created = service
			.create_legal_mention_template(CreateLegalMentionTemplateCommand {
				org_id: OrganizationId(Uuid::new_v4()),
				name: "Mentions légales standard".to_owned(),
				body: "Conformément à la loi…".to_owned(),
			})
			.await
			.unwrap();

		assert!(!created.id.0.is_nil());
		assert!(created.deleted_at.is_none());
	}

	#[tokio::test]
	async fn list_returns_active_templates() {
		let org_id = OrganizationId(Uuid::new_v4());
		let id = LegalMentionTemplateId(Uuid::new_v4());
		let tmpl = template(id);
		let tmpl_clone = tmpl.clone();

		let mut repo = MockLegalMentionTemplateRepository::new();
		repo.expect_list_by_organization()
			.returning(move |_, _, _| {
				let t = tmpl_clone.clone();
				Box::pin(async move { Ok((vec![t], 1u64)) })
			});

		let mut service = LegalMentionTemplateService::new(repo);
		let (items, total) = service
			.list_legal_mention_templates(org_id, 20, 0)
			.await
			.unwrap();

		assert_eq!(total, 1);
		assert_eq!(items.len(), 1);
		assert_eq!(items[0].id, tmpl.id);
	}

	#[tokio::test]
	async fn soft_delete_marks_deleted() {
		let id = LegalMentionTemplateId(Uuid::new_v4());
		let tmpl = template(id);

		let mut repo = MockLegalMentionTemplateRepository::new();
		repo.expect_find_by_id()
			.with(eq(id))
			.returning(move |_| {
				let t = tmpl.clone();
				Box::pin(async move { Ok(Some(t)) })
			});
		repo.expect_soft_delete()
			.with(eq(id), mockall::predicate::always())
			.times(1)
			.returning(|_, _| Box::pin(async { Ok(()) }));

		let mut service = LegalMentionTemplateService::new(repo);
		service
			.soft_delete_legal_mention_template(id)
			.await
			.unwrap();
	}
}
