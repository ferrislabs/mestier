use chrono::Utc;
use common::{CoreError, generate_uuid_v7};

use crate::{
    HostCompany, HostCompanyAddress, HostCompanyId, OrganizationId,
    domain::cursus_host_company::{
        commands::{CreateHostCompanyCommand, HostCompanyFilter, UpdateHostCompanyCommand},
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
        validate_count("employee count", command.employee_count)?;
        validate_count("group employee count", command.group_employee_count)?;
        let email = valid_email(command.email)?;
        let addresses = normalize_addresses(command.addresses)?;

        let now = Utc::now();
        self.repo
            .insert(&HostCompany {
                id: HostCompanyId(generate_uuid_v7()),
                organization_id: command.organization_id,
                name,
                group_name: optional(command.group_name),
                siret: optional(command.siret),
                legal_form: optional(command.legal_form),
                naf_code: optional(command.naf_code),
                capital: optional(command.capital),
                employee_count: command.employee_count,
                group_employee_count: command.group_employee_count,
                director_name: optional(command.director_name),
                phone: optional(command.phone),
                email,
                website: optional(command.website),
                activity_domains: optional(command.activity_domains),
                missions: optional(command.missions),
                addresses,
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
        filter: &HostCompanyFilter,
        limit: u64,
        offset: u64,
    ) -> Result<(Vec<HostCompany>, u64), CoreError> {
        self.repo.list(organization_id, filter, limit, offset).await
    }

    pub async fn update(
        &mut self,
        command: UpdateHostCompanyCommand,
    ) -> Result<HostCompany, CoreError> {
        let name = required("host company name", &command.name)?;
        validate_count("employee count", command.employee_count)?;
        validate_count("group employee count", command.group_employee_count)?;
        let email = valid_email(command.email)?;
        let addresses = normalize_addresses(command.addresses)?;

        let mut company = self.get(command.organization_id, command.id).await?;
        company.name = name;
        company.group_name = optional(command.group_name);
        company.siret = optional(command.siret);
        company.legal_form = optional(command.legal_form);
        company.naf_code = optional(command.naf_code);
        company.capital = optional(command.capital);
        company.employee_count = command.employee_count;
        company.group_employee_count = command.group_employee_count;
        company.director_name = optional(command.director_name);
        company.phone = optional(command.phone);
        company.email = email;
        company.website = optional(command.website);
        company.activity_domains = optional(command.activity_domains);
        company.missions = optional(command.missions);
        company.addresses = addresses;
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

fn validate_count(field: &str, count: Option<i32>) -> Result<(), CoreError> {
    if count.is_some_and(|count| count < 0) {
        return Err(CoreError::Conflict(format!("{field} cannot be negative")));
    }

    Ok(())
}

fn valid_email(value: Option<String>) -> Result<Option<String>, CoreError> {
    let Some(email) = optional(value) else {
        return Ok(None);
    };
    let well_formed = email
        .split_once('@')
        .is_some_and(|(local, domain)| !local.is_empty() && domain.contains('.'))
        && !email.contains(char::is_whitespace);
    if !well_formed {
        return Err(CoreError::Conflict(
            "host company email is not a valid address".to_owned(),
        ));
    }

    Ok(Some(email))
}

fn normalize_addresses(
    addresses: Vec<HostCompanyAddress>,
) -> Result<Vec<HostCompanyAddress>, CoreError> {
    let addresses = addresses
        .into_iter()
        .map(|address| {
            Ok(HostCompanyAddress {
                label: optional(address.label),
                line1: required("host company address line", &address.line1)?,
                line2: optional(address.line2),
                postal_code: optional(address.postal_code),
                city: optional(address.city),
                is_main: address.is_main,
            })
        })
        .collect::<Result<Vec<_>, CoreError>>()?;

    let mains = addresses.iter().filter(|address| address.is_main).count();
    if mains > 1 {
        return Err(CoreError::Conflict(
            "a host company has at most one main address".to_owned(),
        ));
    }
    if mains == 0 && !addresses.is_empty() {
        return Err(CoreError::Conflict(
            "one address of the host company must be the main one".to_owned(),
        ));
    }

    Ok(addresses)
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
            group_name: None,
            siret: None,
            legal_form: None,
            naf_code: None,
            capital: None,
            employee_count: None,
            group_employee_count: None,
            director_name: None,
            phone: None,
            email: None,
            website: None,
            activity_domains: None,
            missions: None,
            addresses: Vec::new(),
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
            group_name: None,
            siret: Some("  ".to_owned()),
            legal_form: None,
            naf_code: None,
            capital: None,
            employee_count: Some(12),
            group_employee_count: None,
            director_name: None,
            phone: None,
            email: None,
            website: None,
            activity_domains: None,
            missions: None,
            addresses: vec![address(" 3 rue des Lilas ", true)],
        }
    }

    fn address(line1: &str, is_main: bool) -> HostCompanyAddress {
        HostCompanyAddress {
            label: Some("  ".to_owned()),
            line1: line1.to_owned(),
            line2: None,
            postal_code: Some(" 69000 ".to_owned()),
            city: Some("Lyon".to_owned()),
            is_main,
        }
    }

    async fn create_with(command: CreateHostCompanyCommand) -> Result<HostCompany, CoreError> {
        let mut repo = MockCursusHostCompanyRepository::new();
        repo.expect_insert().returning(|company| {
            let cloned = company.clone();
            Box::pin(async move { Ok(cloned) })
        });

        CursusHostCompanyService::new(repo).create(command).await
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
        assert_eq!(created.addresses.len(), 1);
        assert_eq!(created.addresses[0].line1, "3 rue des Lilas");
        assert_eq!(created.addresses[0].label, None);
        assert_eq!(created.addresses[0].postal_code.as_deref(), Some("69000"));
        assert!(created.addresses[0].is_main);
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
                group_name: Some("Groupe Durand".to_owned()),
                siret: None,
                legal_form: Some("SAS".to_owned()),
                naf_code: None,
                capital: Some("10 000 EUR".to_owned()),
                employee_count: None,
                group_employee_count: Some(120),
                director_name: Some("M. Durand".to_owned()),
                phone: None,
                email: Some(" contact@durand.fr ".to_owned()),
                website: Some("durand.fr".to_owned()),
                activity_domains: None,
                missions: Some("**Entretien** des espaces verts".to_owned()),
                addresses: vec![address("1 rue A", false), address("2 rue B", true)],
            })
            .await
            .unwrap();

        assert_eq!(updated.name, "Durand SAS");
        assert_eq!(updated.legal_form.as_deref(), Some("SAS"));
        assert_eq!(updated.director_name.as_deref(), Some("M. Durand"));
        assert_eq!(updated.group_name.as_deref(), Some("Groupe Durand"));
        assert_eq!(updated.group_employee_count, Some(120));
        assert_eq!(updated.email.as_deref(), Some("contact@durand.fr"));
        assert_eq!(updated.website.as_deref(), Some("durand.fr"));
        assert_eq!(
            updated.missions.as_deref(),
            Some("**Entretien** des espaces verts")
        );
        let lines: Vec<&str> = updated.addresses.iter().map(|a| a.line1.as_str()).collect();
        assert_eq!(lines, ["1 rue A", "2 rue B"]);
        assert!(!updated.addresses[0].is_main && updated.addresses[1].is_main);
    }

    #[tokio::test]
    async fn create_accepts_a_company_without_any_address() {
        let mut command = create_command(org(), "Durand");
        command.addresses = Vec::new();

        let created = create_with(command).await.unwrap();

        assert!(created.addresses.is_empty());
    }

    #[tokio::test]
    async fn create_rejects_two_main_addresses() {
        let mut command = create_command(org(), "Durand");
        command.addresses = vec![address("1 rue A", true), address("2 rue B", true)];

        assert!(matches!(
            create_with(command).await,
            Err(CoreError::Conflict(_))
        ));
    }

    #[tokio::test]
    async fn create_rejects_addresses_when_none_is_main() {
        let mut command = create_command(org(), "Durand");
        command.addresses = vec![address("1 rue A", false), address("2 rue B", false)];

        assert!(matches!(
            create_with(command).await,
            Err(CoreError::Conflict(_))
        ));
    }

    #[tokio::test]
    async fn create_rejects_a_blank_address_line() {
        let mut command = create_command(org(), "Durand");
        command.addresses = vec![address("   ", true)];

        assert!(matches!(
            create_with(command).await,
            Err(CoreError::Conflict(_))
        ));
    }

    #[tokio::test]
    async fn create_rejects_a_malformed_email_and_treats_a_blank_one_as_absent() {
        let mut bad = create_command(org(), "Durand");
        bad.email = Some("not-an-email".to_owned());
        assert!(matches!(
            create_with(bad).await,
            Err(CoreError::Conflict(_))
        ));

        let mut blank = create_command(org(), "Durand");
        blank.email = Some("  ".to_owned());
        assert_eq!(create_with(blank).await.unwrap().email, None);
    }

    #[tokio::test]
    async fn create_rejects_a_negative_group_employee_count() {
        let mut command = create_command(org(), "Durand");
        command.group_employee_count = Some(-3);

        assert!(matches!(
            create_with(command).await,
            Err(CoreError::Conflict(_))
        ));
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
