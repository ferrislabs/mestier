use chrono::Utc;
use common::{CoreError, generate_uuid_v7};

use crate::{
	OrganizationContext, OrganizationContextId, OrganizationId,
	domain::organization_context::{
		commands::{CreateOrganizationContextCommand, UpdateOrganizationContextCommand},
		ports::OrganizationContextRepository,
	},
};

pub struct OrganizationContextService<R>
where
	R: OrganizationContextRepository,
{
	repo: R,
}

impl<R> OrganizationContextService<R>
where
	R: OrganizationContextRepository,
{
	pub fn new(repo: R) -> Self {
		Self { repo }
	}

	pub async fn create_organization_context(
		&mut self,
		command: CreateOrganizationContextCommand,
	) -> Result<OrganizationContext, CoreError> {
		validate_organization_context(&command.label, &command.address_line, &command.postal_code, &command.city, &command.country, &command.siret, &command.rcs, &command.ape, &command.vat_intracom, &command.iban, &command.bic)?;

		let now = Utc::now();
		self.repo
			.insert(&OrganizationContext {
				id: OrganizationContextId(generate_uuid_v7()),
				org_id: command.org_id,
				label: command.label,
				address_line: command.address_line,
				postal_code: command.postal_code,
				city: command.city,
				country: command.country,
				siret: command.siret,
				rcs: command.rcs,
				ape: command.ape,
				vat_intracom: command.vat_intracom,
				iban: command.iban,
				bic: command.bic,
				deleted_at: None,
				created_at: now,
				updated_at: now,
			})
			.await
	}

	pub async fn get_organization_context(
		&mut self,
		id: OrganizationContextId,
	) -> Result<OrganizationContext, CoreError> {
		self.repo.find_by_id(id).await?.ok_or(CoreError::NotFound)
	}

	pub async fn list_organization_contexts(
		&mut self,
		org_id: OrganizationId,
		limit: u64,
		offset: u64,
	) -> Result<(Vec<OrganizationContext>, u64), CoreError> {
		self.repo.list_by_organization(org_id, limit, offset).await
	}

	pub async fn update_organization_context(
		&mut self,
		command: UpdateOrganizationContextCommand,
	) -> Result<OrganizationContext, CoreError> {
		validate_organization_context(&command.label, &command.address_line, &command.postal_code, &command.city, &command.country, &command.siret, &command.rcs, &command.ape, &command.vat_intracom, &command.iban, &command.bic)?;

		let mut organization_context = self.get_organization_context(command.id).await?;
		organization_context.label = command.label;
		organization_context.address_line = command.address_line;
		organization_context.postal_code = command.postal_code;
		organization_context.city = command.city;
		organization_context.country = command.country;
		organization_context.siret = command.siret;
		organization_context.rcs = command.rcs;
		organization_context.ape = command.ape;
		organization_context.vat_intracom = command.vat_intracom;
		organization_context.iban = command.iban;
		organization_context.bic = command.bic;
		organization_context.updated_at = Utc::now();

		self.repo.update(&organization_context).await
	}

	pub async fn soft_delete_organization_context(
		&mut self,
		id: OrganizationContextId,
	) -> Result<(), CoreError> {
		self.get_organization_context(id).await?;
		self.repo.soft_delete(id, Utc::now()).await
	}
}

#[allow(clippy::too_many_arguments)]
fn validate_organization_context(
	label: &str,
	address_line: &Option<String>,
	postal_code: &Option<String>,
	city: &Option<String>,
	country: &Option<String>,
	siret: &Option<String>,
	rcs: &Option<String>,
	ape: &Option<String>,
	vat_intracom: &Option<String>,
	iban: &Option<String>,
	bic: &Option<String>,
) -> Result<(), CoreError> {
	validate_required("organization_context label", label)?;
	validate_optional("organization_context address line", address_line)?;
	validate_optional("organization_context postal code", postal_code)?;
	validate_optional("organization_context city", city)?;
	validate_optional("organization_context country", country)?;
	validate_optional("organization_context siret", siret)?;
	validate_optional("organization_context rcs", rcs)?;
	validate_optional("organization_context ape", ape)?;
	validate_optional("organization_context vat_intracom", vat_intracom)?;
	validate_optional("organization_context iban", iban)?;
	validate_optional("organization_context bic", bic)?;
	Ok(())
}

fn validate_required(label: &str, value: &str) -> Result<(), CoreError> {
	if value.trim().is_empty() {
		return Err(CoreError::Conflict(format!("{label} cannot be empty")));
	}
	Ok(())
}

fn validate_optional(label: &str, value: &Option<String>) -> Result<(), CoreError> {
	if value.as_deref().is_some_and(|v| v.trim().is_empty()) {
		return Err(CoreError::Conflict(format!("{label} cannot be empty")));
	}
	Ok(())
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::domain::organization_context::ports::MockOrganizationContextRepository;
	use mockall::predicate::eq;
	use uuid::Uuid;

	fn organization_context(id: OrganizationContextId) -> OrganizationContext {
		let now = Utc::now();
		OrganizationContext {
			id,
			org_id: OrganizationId(Uuid::new_v4()),
			label: "Siège social".to_owned(),
			address_line: Some("1 rue de la Paix".to_owned()),
			postal_code: Some("75001".to_owned()),
			city: Some("Paris".to_owned()),
			country: Some("France".to_owned()),
			siret: Some("12345678901234".to_owned()),
			rcs: None,
			ape: None,
			vat_intracom: None,
			iban: None,
			bic: None,
			deleted_at: None,
			created_at: now,
			updated_at: now,
		}
	}

	#[tokio::test]
	async fn create_organization_context_persists_via_repo() {
		let mut repo = MockOrganizationContextRepository::new();
		repo.expect_insert().times(1).returning(|p| {
			let ctx = p.clone();
			Box::pin(async move { Ok(ctx) })
		});

		let mut service = OrganizationContextService::new(repo);
		let created = service
			.create_organization_context(CreateOrganizationContextCommand {
				org_id: OrganizationId(Uuid::new_v4()),
				label: "Siège social".to_owned(),
				address_line: Some("1 rue de la Paix".to_owned()),
				postal_code: Some("75001".to_owned()),
				city: Some("Paris".to_owned()),
				country: Some("France".to_owned()),
				siret: Some("12345678901234".to_owned()),
				rcs: None,
				ape: None,
				vat_intracom: None,
				iban: None,
				bic: None,
			})
			.await
			.unwrap();

		assert_eq!(created.label, "Siège social");
	}

	#[tokio::test]
	async fn list_organization_contexts_delegates_to_repo() {
		let org_id = OrganizationId(Uuid::new_v4());
		let mut repo = MockOrganizationContextRepository::new();
		repo.expect_list_by_organization()
			.with(eq(org_id), eq(10), eq(0))
			.returning(move |_, _, _| {
				Box::pin(async move {
					Ok((
						vec![organization_context(OrganizationContextId(Uuid::new_v4()))],
						1,
					))
				})
			});

		let mut service = OrganizationContextService::new(repo);
		let (items, total) = service
			.list_organization_contexts(org_id, 10, 0)
			.await
			.unwrap();

		assert_eq!(items.len(), 1);
		assert_eq!(total, 1);
	}

	#[tokio::test]
	async fn soft_delete_organization_context_checks_existence_then_deletes() {
		let id = OrganizationContextId(Uuid::new_v4());
		let mut repo = MockOrganizationContextRepository::new();
		repo.expect_find_by_id()
			.with(eq(id))
			.returning(move |_| Box::pin(async move { Ok(Some(organization_context(id))) }));
		repo.expect_soft_delete()
			.withf(move |deleted_id, _| *deleted_id == id)
			.returning(|_, _| Box::pin(async { Ok(()) }));

		let mut service = OrganizationContextService::new(repo);
		service.soft_delete_organization_context(id).await.unwrap();
	}

	#[tokio::test]
	async fn create_organization_context_rejects_blank_label() {
		let repo = MockOrganizationContextRepository::new();
		let mut service = OrganizationContextService::new(repo);

		let err = service
			.create_organization_context(CreateOrganizationContextCommand {
				org_id: OrganizationId(Uuid::new_v4()),
				label: "   ".to_owned(),
				address_line: None,
				postal_code: None,
				city: None,
				country: None,
				siret: None,
				rcs: None,
				ape: None,
				vat_intracom: None,
				iban: None,
				bic: None,
			})
			.await
			.unwrap_err();

		assert!(matches!(err, CoreError::Conflict(_)));
	}

	#[tokio::test]
	async fn create_organization_context_rejects_blank_optional_fields() {
		let repo = MockOrganizationContextRepository::new();
		let mut service = OrganizationContextService::new(repo);

		let err = service
			.create_organization_context(CreateOrganizationContextCommand {
				org_id: OrganizationId(Uuid::new_v4()),
				label: "Siège social".to_owned(),
				address_line: Some("".to_owned()),
				postal_code: None,
				city: None,
				country: None,
				siret: None,
				rcs: None,
				ape: None,
				vat_intracom: None,
				iban: None,
				bic: None,
			})
			.await
			.unwrap_err();

		assert!(matches!(err, CoreError::Conflict(_)));
	}
}
