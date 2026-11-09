use chrono::Utc;
use common::{CoreError, generate_uuid_v7};

use crate::{
    HostCompany, HostCompanyId, OrganizationId,
    domain::cursus_host_company::{
        commands::{CreateHostCompanyCommand, UpdateHostCompanyCommand},
        ports::CursusHostCompanyRepository,
    },
};

pub struct CursusHostCompanyService<R>
where
    R: CursusHostCompanyRepository,
{
    repo: R,
}

impl<R> CursusHostCompanyService<R>
where
    R: CursusHostCompanyRepository,
{
    pub fn new(repo: R) -> Self {
        Self { repo }
    }

    pub async fn create(
        &mut self,
        command: CreateHostCompanyCommand,
    ) -> Result<HostCompany, CoreError> {
        let name = required("host company name", &command.name)?;
        validate_employee_count(command.employee_count)?;

        let now = Utc::now();
        self.repo
            .insert(&HostCompany {
                id: HostCompanyId(generate_uuid_v7()),
                organization_id: command.organization_id,
                name,
                siret: optional(command.siret),
                address: optional(command.address),
                legal_form: optional(command.legal_form),
                naf_code: optional(command.naf_code),
                employee_count: command.employee_count,
                director_name: optional(command.director_name),
                archived_at: None,
                created_at: now,
                updated_at: now,
            })
            .await
    }

    pub async fn get(
        &mut self,
        organization_id: OrganizationId,
        id: HostCompanyId,
    ) -> Result<HostCompany, CoreError> {
        self.repo
            .find(organization_id, id)
            .await?
            .filter(|found| found.organization_id == organization_id)
            .ok_or(CoreError::NotFound)
    }

    pub async fn list(
        &mut self,
        organization_id: OrganizationId,
        include_archived: bool,
        limit: u64,
        offset: u64,
    ) -> Result<(Vec<HostCompany>, u64), CoreError> {
        self.repo
            .list(organization_id, include_archived, limit, offset)
            .await
    }

    pub async fn update(
        &mut self,
        command: UpdateHostCompanyCommand,
    ) -> Result<HostCompany, CoreError> {
        let name = required("host company name", &command.name)?;
        validate_employee_count(command.employee_count)?;

        let mut company = self.get(command.organization_id, command.id).await?;
        company.name = name;
        company.siret = optional(command.siret);
        company.address = optional(command.address);
        company.legal_form = optional(command.legal_form);
        company.naf_code = optional(command.naf_code);
        company.employee_count = command.employee_count;
        company.director_name = optional(command.director_name);
        company.updated_at = Utc::now();

        self.repo.update(&company).await
    }

    pub async fn archive(
        &mut self,
        organization_id: OrganizationId,
        id: HostCompanyId,
    ) -> Result<HostCompany, CoreError> {
        self.get(organization_id, id).await?;
        self.repo.archive(organization_id, id, Utc::now()).await
    }
}

fn required(field: &str, value: &str) -> Result<String, CoreError> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(CoreError::Conflict(format!("{field} cannot be empty")));
    }

    Ok(trimmed.to_owned())
}

fn optional(value: Option<String>) -> Option<String> {
    value
        .map(|text| text.trim().to_owned())
        .filter(|text| !text.is_empty())
}

fn validate_employee_count(count: Option<i32>) -> Result<(), CoreError> {
    if count.is_some_and(|count| count < 0) {
        return Err(CoreError::Conflict(
            "employee count cannot be negative".to_owned(),
        ));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::cursus_host_company::ports::MockCursusHostCompanyRepository;
    use mockall::predicate::eq;
    use uuid::Uuid;

    fn org() -> OrganizationId {
        OrganizationId(Uuid::new_v4())
    }

    fn actor() -> authz::Subject {
        crate::application::policy::user_subject(crate::UserId(Uuid::new_v4()), Vec::new())
    }

    fn company_in(organization_id: OrganizationId, id: HostCompanyId) -> HostCompany {
        let now = Utc::now();
        HostCompany {
            id,
            organization_id,
            name: "Paysages Durand".to_owned(),
            siret: None,
            address: None,
            legal_form: None,
            naf_code: None,
            employee_count: None,
            director_name: None,
            archived_at: None,
            created_at: now,
            updated_at: now,
        }
    }

    fn create_command(organization_id: OrganizationId, name: &str) -> CreateHostCompanyCommand {
        CreateHostCompanyCommand {
            actor: actor(),
            organization_id,
            name: name.to_owned(),
            siret: Some("  ".to_owned()),
            address: Some(" 3 rue des Lilas ".to_owned()),
            legal_form: None,
            naf_code: None,
            employee_count: Some(12),
            director_name: None,
        }
    }

    #[tokio::test]
    async fn create_trims_the_name_and_turns_blank_optionals_into_none() {
        let organization_id = org();
        let mut repo = MockCursusHostCompanyRepository::new();
        repo.expect_insert().times(1).returning(|company| {
            let cloned = company.clone();
            Box::pin(async move { Ok(cloned) })
        });

        let created = CursusHostCompanyService::new(repo)
            .create(create_command(organization_id, "  Durand  "))
            .await
            .unwrap();

        assert_eq!(created.name, "Durand");
        assert_eq!(created.siret, None);
        assert_eq!(created.address.as_deref(), Some("3 rue des Lilas"));
        assert_eq!(created.organization_id, organization_id);
        assert!(created.archived_at.is_none());
    }

    #[tokio::test]
    async fn create_rejects_a_blank_name() {
        let result = CursusHostCompanyService::new(MockCursusHostCompanyRepository::new())
            .create(create_command(org(), "   "))
            .await;

        assert!(matches!(result, Err(CoreError::Conflict(_))));
    }

    #[tokio::test]
    async fn create_rejects_a_negative_employee_count() {
        let mut command = create_command(org(), "Durand");
        command.employee_count = Some(-1);

        let result = CursusHostCompanyService::new(MockCursusHostCompanyRepository::new())
            .create(command)
            .await;

        assert!(matches!(result, Err(CoreError::Conflict(_))));
    }

    #[tokio::test]
    async fn get_hides_a_company_of_another_organization() {
        let asked = org();
        let id = HostCompanyId(Uuid::new_v4());
        let mut repo = MockCursusHostCompanyRepository::new();
        repo.expect_find()
            .with(eq(asked), eq(id))
            .returning(move |_, id| {
                let foreign = company_in(org(), id);
                Box::pin(async move { Ok(Some(foreign)) })
            });

        let result = CursusHostCompanyService::new(repo).get(asked, id).await;

        assert!(matches!(result, Err(CoreError::NotFound)));
    }

    #[tokio::test]
    async fn update_replaces_every_editable_field() {
        let organization_id = org();
        let id = HostCompanyId(Uuid::new_v4());
        let mut repo = MockCursusHostCompanyRepository::new();
        repo.expect_find().returning(move |organization_id, id| {
            let existing = company_in(organization_id, id);
            Box::pin(async move { Ok(Some(existing)) })
        });
        repo.expect_update().times(1).returning(|company| {
            let cloned = company.clone();
            Box::pin(async move { Ok(cloned) })
        });

        let updated = CursusHostCompanyService::new(repo)
            .update(UpdateHostCompanyCommand {
                actor: actor(),
                organization_id,
                id,
                name: "Durand SAS".to_owned(),
                siret: None,
                address: None,
                legal_form: Some("SAS".to_owned()),
                naf_code: None,
                employee_count: None,
                director_name: Some("M. Durand".to_owned()),
            })
            .await
            .unwrap();

        assert_eq!(updated.name, "Durand SAS");
        assert_eq!(updated.legal_form.as_deref(), Some("SAS"));
        assert_eq!(updated.director_name.as_deref(), Some("M. Durand"));
    }

    #[tokio::test]
    async fn archive_of_an_unknown_company_is_not_found() {
        let mut repo = MockCursusHostCompanyRepository::new();
        repo.expect_find()
            .returning(|_, _| Box::pin(async { Ok(None) }));

        let result = CursusHostCompanyService::new(repo)
            .archive(org(), HostCompanyId(Uuid::new_v4()))
            .await;

        assert!(matches!(result, Err(CoreError::NotFound)));
    }
}
