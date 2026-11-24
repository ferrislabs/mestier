use chrono::Utc;
use common::{CoreError, generate_uuid_v7};

use crate::{
    Customer, Daf, DafAttachment, DafDesignation, DafDossier, DafId, DafNumber, DafStatus,
    DafSummary, DoeStyle, Organization, OrganizationId, Project, ProjectId, SupplierSnapshot,
    domain::{
        customer::ports::CustomerRepository,
        daf::{
            MAX_ATTACHMENTS, MAX_DAFS_PER_PROJECT,
            commands::{
                CreateDafCommand, PlaceDafPhotoCommand, UpdateDafCommand, UpdateDafDossierCommand,
            },
            ports::DafRepository,
            render::DafDocument,
        },
        doe::{PartyRole, ports::DoeRepository},
        doe_model::{
            ports::{DoeModelRepository, DoeSettingsRepository},
            service::inherited_style,
        },
        file_storage::{ports::FileStorage, service::FileStorageService},
        organization::ports::OrganizationRepository,
        photo_library::{ports::LibraryPhotoRepository, service::PhotoLibraryService},
        product::ports::ProductRepository,
        project::ports::ProjectRepository,
        supplier_invoice::ports::SupplierRepository,
    },
};

fn blank_to_none(value: Option<String>) -> Option<String> {
    value
        .map(|text| text.trim().to_owned())
        .filter(|text| !text.is_empty())
}

fn required(value: &str, what: &str) -> Result<String, CoreError> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(CoreError::Conflict(format!("{what} cannot be blank")));
    }
    Ok(trimmed.to_owned())
}

fn clean_supplier(supplier: SupplierSnapshot) -> Result<SupplierSnapshot, CoreError> {
    Ok(SupplierSnapshot {
        name: required(&supplier.name, "a supplier name")?,
        address: blank_to_none(supplier.address),
        phone: blank_to_none(supplier.phone),
        email: blank_to_none(supplier.email),
    })
}

fn clean_attachment(attachment: DafAttachment) -> Result<DafAttachment, CoreError> {
    Ok(DafAttachment {
        key: required(&attachment.key, "an attachment key")?,
        name: required(&attachment.name, "an attachment name")?,
    })
}

fn installer_of(organization: &Organization) -> Option<String> {
    let locality = [
        organization.address_postal_code.as_deref(),
        organization.address_city.as_deref(),
    ]
    .into_iter()
    .flatten()
    .map(str::trim)
    .filter(|text| !text.is_empty())
    .collect::<Vec<_>>()
    .join(" ");
    let lines = [
        Some(organization.name.trim().to_owned()),
        organization.address_line1.clone(),
        organization.address_line2.clone(),
        Some(locality),
    ]
    .into_iter()
    .filter_map(blank_to_none)
    .collect::<Vec<_>>()
    .join("\n");
    (!lines.is_empty()).then_some(lines)
}

fn attachment_name(index: usize) -> String {
    match index {
        0 => "Fiche technique".to_owned(),
        _ => format!("Fiche technique {}", index + 1),
    }
}

/// The repositories a dossier is prefilled from, and a document's style
/// chain is read through.
pub struct DafLookups<'a, PR, OR, CR, DR> {
    pub projects: &'a mut PR,
    pub organizations: &'a mut OR,
    pub customers: &'a mut CR,
    pub does: &'a mut DR,
}

impl<PR, OR, CR, DR> DafLookups<'_, PR, OR, CR, DR>
where
    PR: ProjectRepository,
    OR: OrganizationRepository,
    CR: CustomerRepository,
    DR: DoeRepository,
{
    async fn project(
        &mut self,
        organization_id: OrganizationId,
        project_id: ProjectId,
    ) -> Result<Project, CoreError> {
        self.projects
            .find_by_id(project_id)
            .await?
            .filter(|project| project.organization_id == organization_id)
            .ok_or(CoreError::NotFound)
    }

    async fn organization(
        &mut self,
        organization_id: OrganizationId,
    ) -> Result<Organization, CoreError> {
        self.organizations
            .find_by_id(organization_id)
            .await?
            .ok_or(CoreError::NotFound)
    }

    async fn prefill(&mut self, project: &Project) -> Result<DafDossier, CoreError> {
        let organization = self.organization(project.organization_id).await?;
        let owner: Option<Customer> = match project.customer_id {
            Some(customer_id) => self
                .customers
                .find_by_id(customer_id)
                .await?
                .filter(|customer| customer.organization_id == project.organization_id),
            None => None,
        };
        let manager = self
            .does
            .find_by_project(project.id)
            .await?
            .filter(|doe| doe.organization_id == project.organization_id)
            .and_then(|doe| {
                doe.parties
                    .into_iter()
                    .find(|party| party.role == PartyRole::ProjectManager)
            })
            .and_then(|party| blank_to_none(Some(party.name)));

        Ok(DafDossier {
            organization_id: project.organization_id,
            project_id: project.id,
            title: project.name.clone(),
            issuer: organization.name,
            project_owner: owner.and_then(|customer| blank_to_none(Some(customer.name))),
            project_manager: manager,
            updated_at: None,
        })
    }

    async fn style<MR, SR>(
        &mut self,
        organization_id: OrganizationId,
        project_id: ProjectId,
        models: &mut MR,
        settings: &mut SR,
    ) -> Result<DoeStyle, CoreError>
    where
        MR: DoeModelRepository,
        SR: DoeSettingsRepository,
    {
        if let Some(style) = settings
            .find_project_style(organization_id, project_id)
            .await?
        {
            return Ok(style);
        }
        let model_id = self
            .does
            .find_by_project(project_id)
            .await?
            .filter(|doe| doe.organization_id == organization_id)
            .and_then(|doe| doe.model_id);
        inherited_style(models, settings, organization_id, model_id).await
    }
}

pub struct DafService<R>
where
    R: DafRepository,
{
    repository: R,
}

impl<R> DafService<R>
where
    R: DafRepository,
{
    pub fn new(repository: R) -> Self {
        Self { repository }
    }

    pub async fn get(&mut self, id: DafId) -> Result<Daf, CoreError> {
        self.repository
            .find_by_id(id)
            .await?
            .ok_or(CoreError::NotFound)
    }

    /// The DAF, only when it belongs to the organization.
    pub async fn get_in(
        &mut self,
        organization_id: OrganizationId,
        id: DafId,
    ) -> Result<Daf, CoreError> {
        let daf = self.get(id).await?;
        if daf.organization_id != organization_id {
            return Err(CoreError::NotFound);
        }
        Ok(daf)
    }

    /// The summary rows of a project the caller has already checked belongs
    /// to its organization.
    pub async fn summaries(&mut self, project_id: ProjectId) -> Result<Vec<DafSummary>, CoreError> {
        self.repository.list_summaries(project_id).await
    }

    /// The stored dossier, else the prefilled one, which is not saved.
    pub async fn dossier<PR, OR, CR, DR>(
        &mut self,
        organization_id: OrganizationId,
        project_id: ProjectId,
        lookups: &mut DafLookups<'_, PR, OR, CR, DR>,
    ) -> Result<DafDossier, CoreError>
    where
        PR: ProjectRepository,
        OR: OrganizationRepository,
        CR: CustomerRepository,
        DR: DoeRepository,
    {
        let project = lookups.project(organization_id, project_id).await?;
        match self.repository.find_dossier(project_id).await? {
            Some(dossier) if dossier.organization_id == organization_id => Ok(dossier),
            Some(_) => Err(CoreError::NotFound),
            None => lookups.prefill(&project).await,
        }
    }

    pub async fn save_dossier<PR, OR, CR, DR>(
        &mut self,
        command: UpdateDafDossierCommand,
        lookups: &mut DafLookups<'_, PR, OR, CR, DR>,
    ) -> Result<DafDossier, CoreError>
    where
        PR: ProjectRepository,
        OR: OrganizationRepository,
        CR: CustomerRepository,
        DR: DoeRepository,
    {
        lookups
            .project(command.organization_id, command.project_id)
            .await?;
        let dossier = DafDossier {
            organization_id: command.organization_id,
            project_id: command.project_id,
            title: required(&command.title, "the DAF dossier title")?,
            issuer: required(&command.issuer, "the DAF dossier issuer")?,
            project_owner: blank_to_none(command.project_owner),
            project_manager: blank_to_none(command.project_manager),
            updated_at: None,
        };
        self.repository.upsert_dossier(&dossier).await
    }

    pub async fn create<PR, OR, CR, DR, PD, SR>(
        &mut self,
        command: CreateDafCommand,
        lookups: &mut DafLookups<'_, PR, OR, CR, DR>,
        products: &mut PD,
        suppliers: &mut SR,
    ) -> Result<Daf, CoreError>
    where
        PR: ProjectRepository,
        OR: OrganizationRepository,
        CR: CustomerRepository,
        DR: DoeRepository,
        PD: ProductRepository,
        SR: SupplierRepository,
    {
        if command.designation.is_none() && command.product_id.is_none() {
            return Err(CoreError::Conflict(
                "a DAF needs a designation or a product".to_owned(),
            ));
        }
        let organization_id = command.organization_id;
        let dossier = self
            .dossier(organization_id, command.project_id, lookups)
            .await?;

        let existing = self.repository.list_summaries(command.project_id).await?;
        if existing.len() >= MAX_DAFS_PER_PROJECT {
            return Err(CoreError::Conflict(format!(
                "a project holds at most {MAX_DAFS_PER_PROJECT} DAF"
            )));
        }
        let number = match existing.iter().map(|summary| summary.number).max() {
            None => DafNumber::try_from(1)?,
            Some(last) => last.next().ok_or_else(|| {
                CoreError::Conflict(format!("no DAF number left past {}", DafNumber::MAX))
            })?,
        };

        let organization = lookups.organization(organization_id).await?;
        let product = match command.product_id {
            Some(product_id) => Some(
                products
                    .find_by_id(product_id)
                    .await?
                    .filter(|product| {
                        product.organization_id == organization_id && product.deleted_at.is_none()
                    })
                    .ok_or(CoreError::NotFound)?,
            ),
            None => None,
        };
        let designation = match (command.designation, product.as_ref()) {
            (Some(designation), _) => designation,
            (None, Some(product)) => DafDesignation::try_from(product.name.clone())?,
            (None, None) => {
                return Err(CoreError::Conflict(
                    "a DAF needs a designation or a product".to_owned(),
                ));
            }
        };

        let now = Utc::now();
        let mut daf = Daf {
            id: DafId(generate_uuid_v7()),
            organization_id,
            project_id: command.project_id,
            number,
            designation,
            price_ref: None,
            source_product_id: None,
            supplier: None,
            installer: installer_of(&organization),
            specs: None,
            observations: None,
            photo_key: None,
            attachments: Vec::new(),
            project_owner: None,
            project_manager: None,
            status: DafStatus::Pending,
            sent_on: None,
            issued_on: now.date_naive(),
            returned_on: None,
            review_note: None,
            created_at: now,
            updated_at: now,
        };

        if let Some(product) = product {
            let supplier = match product.supplier_id {
                Some(supplier_id) => suppliers
                    .find_by_id(supplier_id)
                    .await?
                    .filter(|supplier| supplier.organization_id == organization_id),
                None => None,
            };
            daf.source_product_id = Some(product.id);
            daf.supplier = supplier.map(|supplier| SupplierSnapshot {
                name: supplier.name,
                address: supplier.address,
                phone: supplier.phone,
                email: supplier.email,
            });
            daf.specs = blank_to_none(product.technical_specs);
            daf.photo_key = product.photo_keys.into_iter().next();
            daf.attachments = product
                .datasheet_keys
                .into_iter()
                .take(MAX_ATTACHMENTS)
                .enumerate()
                .map(|(index, key)| DafAttachment {
                    key,
                    name: attachment_name(index),
                })
                .collect();
        }

        if dossier.updated_at.is_none() {
            self.repository.upsert_dossier(&dossier).await?;
        }
        self.repository.insert(&daf).await
    }

    pub async fn replace(&mut self, command: UpdateDafCommand) -> Result<Daf, CoreError> {
        let existing = self.get(command.id).await?;

        if command
            .expected_updated_at
            .is_some_and(|expected| expected != existing.updated_at)
        {
            return Err(CoreError::Conflict(
                "la DAF a été modifiée entre-temps, relis-la avant de l'enregistrer".to_owned(),
            ));
        }
        if command.attachments.len() > MAX_ATTACHMENTS {
            return Err(CoreError::Conflict(format!(
                "a DAF holds at most {MAX_ATTACHMENTS} attachments"
            )));
        }

        let daf = Daf {
            number: command.number,
            designation: command.designation,
            price_ref: blank_to_none(command.price_ref),
            supplier: command.supplier.map(clean_supplier).transpose()?,
            installer: blank_to_none(command.installer),
            specs: blank_to_none(command.specs),
            observations: blank_to_none(command.observations),
            photo_key: blank_to_none(command.photo_key),
            attachments: command
                .attachments
                .into_iter()
                .map(clean_attachment)
                .collect::<Result<_, _>>()?,
            project_owner: blank_to_none(command.project_owner),
            project_manager: blank_to_none(command.project_manager),
            status: command.status,
            sent_on: command.sent_on,
            issued_on: command.issued_on,
            returned_on: command.returned_on,
            review_note: blank_to_none(command.review_note),
            updated_at: Utc::now(),
            ..existing
        };

        self.repository.update(&daf).await
    }

    pub async fn delete(&mut self, id: DafId) -> Result<(), CoreError> {
        self.repository.delete(id).await
    }

    /// Copies the library photo to a file of its own and makes it the DAF's
    /// image.
    pub async fn place_photo<LR, S>(
        &mut self,
        command: PlaceDafPhotoCommand,
        photos: &mut PhotoLibraryService<LR>,
        storage: &FileStorageService<S>,
    ) -> Result<Daf, CoreError>
    where
        LR: LibraryPhotoRepository,
        S: FileStorage,
    {
        let existing = self.get_in(command.organization_id, command.id).await?;
        let key = photos
            .copy_file(command.organization_id, command.photo_id, storage)
            .await?;
        let daf = Daf {
            photo_key: Some(key),
            updated_at: Utc::now(),
            ..existing
        };
        self.repository.update(&daf).await
    }

    /// One DAF as it will be printed, in its project's style.
    pub async fn document_single<PR, OR, CR, DR, MR, SR>(
        &mut self,
        organization_id: OrganizationId,
        id: DafId,
        lookups: &mut DafLookups<'_, PR, OR, CR, DR>,
        models: &mut MR,
        settings: &mut SR,
    ) -> Result<DafDocument, CoreError>
    where
        PR: ProjectRepository,
        OR: OrganizationRepository,
        CR: CustomerRepository,
        DR: DoeRepository,
        MR: DoeModelRepository,
        SR: DoeSettingsRepository,
    {
        let daf = self.get_in(organization_id, id).await?;
        let dossier = self
            .dossier(organization_id, daf.project_id, lookups)
            .await?;
        let style = lookups
            .style(organization_id, daf.project_id, models, settings)
            .await?;
        Ok(DafDocument::single(&dossier, &daf, &style))
    }

    /// The summary then every DAF of the project, in its style.
    pub async fn document_dossier<PR, OR, CR, DR, MR, SR>(
        &mut self,
        organization_id: OrganizationId,
        project_id: ProjectId,
        lookups: &mut DafLookups<'_, PR, OR, CR, DR>,
        models: &mut MR,
        settings: &mut SR,
    ) -> Result<DafDocument, CoreError>
    where
        PR: ProjectRepository,
        OR: OrganizationRepository,
        CR: CustomerRepository,
        DR: DoeRepository,
        MR: DoeModelRepository,
        SR: DoeSettingsRepository,
    {
        let dossier = self.dossier(organization_id, project_id, lookups).await?;
        let dafs = self.repository.list_by_project(project_id).await?;
        let style = lookups
            .style(organization_id, project_id, models, settings)
            .await?;
        Ok(DafDocument::dossier(
            &dossier,
            &dafs,
            &style,
            Utc::now().date_naive(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use chrono::NaiveDate;
    use uuid::Uuid;

    use super::*;
    use crate::{
        CustomerId, Doe, DoeId, DoeParty, Product, ProductId, Revision, Supplier, SupplierId,
        domain::{
            customer::ports::MockCustomerRepository, daf::ports::MockDafRepository,
            doe::ports::MockDoeRepository, organization::ports::MockOrganizationRepository,
            product::ports::MockProductRepository, project::ports::MockProjectRepository,
            supplier_invoice::ports::MockSupplierRepository,
        },
    };

    struct Fixture {
        organization_id: OrganizationId,
        project: Project,
        customer: Customer,
        organization: Organization,
    }

    fn fixture() -> Fixture {
        let now = Utc::now();
        let organization_id = OrganizationId(Uuid::new_v4());
        let customer = Customer {
            id: CustomerId(Uuid::new_v4()),
            organization_id,
            status: crate::CustomerStatus::Client,
            name: "Commune de Cébazat".to_owned(),
            registration_number: None,
            phone: None,
            email: None,
            profile: crate::CustomerProfile::default(),
            converted_at: None,
            archived_at: None,
            deleted_at: None,
            created_at: now,
            updated_at: now,
        };
        let project = Project {
            id: ProjectId(Uuid::new_v4()),
            organization_id,
            name: "Parc Montgroux".to_owned(),
            customer_id: Some(customer.id),
            customer_context_id: None,
            quote_id: None,
            archived_at: None,
            created_at: now,
            updated_at: now,
        };
        let organization = Organization {
            id: organization_id,
            name: "Treyve Paysages".to_owned(),
            slug: "treyve".to_owned(),
            owner_id: crate::UserId(Uuid::new_v4()),
            legal_name: None,
            legal_form: None,
            registration_number: None,
            vat_status: None,
            share_capital_cents: None,
            address_line1: Some("3 chemin des Vignes".to_owned()),
            address_line2: None,
            address_postal_code: Some("03120".to_owned()),
            address_city: Some("Lapalisse".to_owned()),
            address_country: None,
            contact_email: None,
            contact_phone: None,
            insurance_mention: None,
            quote_number_prefix: "DEV".to_owned(),
            invoice_number_prefix: "FAC".to_owned(),
            field_clock_enabled: false,
            vat_on_debits: false,
            mcp_enabled: false,
            deleted_at: None,
            created_at: now,
            updated_at: now,
        };
        Fixture {
            organization_id,
            project,
            customer,
            organization,
        }
    }

    struct Mocks {
        projects: MockProjectRepository,
        organizations: MockOrganizationRepository,
        customers: MockCustomerRepository,
        does: MockDoeRepository,
    }

    fn mocks(fixture: &Fixture, doe: Option<Doe>) -> Mocks {
        let project = fixture.project.clone();
        let mut projects = MockProjectRepository::new();
        projects.expect_find_by_id().returning(move |_| {
            let project = project.clone();
            Box::pin(async move { Ok(Some(project)) })
        });
        let organization = fixture.organization.clone();
        let mut organizations = MockOrganizationRepository::new();
        organizations.expect_find_by_id().returning(move |_| {
            let organization = organization.clone();
            Box::pin(async move { Ok(Some(organization)) })
        });
        let customer = fixture.customer.clone();
        let mut customers = MockCustomerRepository::new();
        customers.expect_find_by_id().returning(move |_| {
            let customer = customer.clone();
            Box::pin(async move { Ok(Some(customer)) })
        });
        let mut does = MockDoeRepository::new();
        does.expect_find_by_project().returning(move |_| {
            let doe = doe.clone();
            Box::pin(async move { Ok(doe) })
        });
        Mocks {
            projects,
            organizations,
            customers,
            does,
        }
    }

    fn actor() -> authz::Subject {
        authz::Subject::system()
    }

    fn summary(number: u16) -> DafSummary {
        let now = Utc::now();
        DafSummary {
            id: DafId(Uuid::new_v4()),
            number: DafNumber::try_from(number).unwrap(),
            designation: DafDesignation::try_from(format!("DAF {number}")).unwrap(),
            price_ref: None,
            status: DafStatus::Pending,
            sent_on: None,
            issued_on: now.date_naive(),
            returned_on: None,
            review_note: None,
            updated_at: now,
        }
    }

    fn repository(fixture: &Fixture, dossier_stored: bool, numbers: Vec<u16>) -> MockDafRepository {
        let mut repository = MockDafRepository::new();
        let stored = DafDossier {
            organization_id: fixture.organization_id,
            project_id: fixture.project.id,
            title: "Stocké".to_owned(),
            issuer: "Moi".to_owned(),
            project_owner: None,
            project_manager: None,
            updated_at: Some(Utc::now()),
        };
        repository.expect_find_dossier().returning(move |_| {
            let stored = dossier_stored.then(|| stored.clone());
            Box::pin(async move { Ok(stored) })
        });
        repository.expect_upsert_dossier().returning(|dossier| {
            let dossier = dossier.clone();
            Box::pin(async move { Ok(dossier) })
        });
        repository.expect_list_summaries().returning(move |_| {
            let rows: Vec<DafSummary> = numbers.iter().map(|n| summary(*n)).collect();
            Box::pin(async move { Ok(rows) })
        });
        repository.expect_insert().returning(|daf| {
            let daf = daf.clone();
            Box::pin(async move { Ok(daf) })
        });
        repository
    }

    fn create_command(fixture: &Fixture, designation: Option<&str>) -> CreateDafCommand {
        CreateDafCommand {
            actor: actor(),
            organization_id: fixture.organization_id,
            project_id: fixture.project.id,
            designation: designation.map(|text| DafDesignation::try_from(text.to_owned()).unwrap()),
            product_id: None,
        }
    }

    async fn create(
        fixture: &Fixture,
        repository: MockDafRepository,
        command: CreateDafCommand,
        products: MockProductRepository,
        suppliers: MockSupplierRepository,
    ) -> Result<Daf, CoreError> {
        let mut m = mocks(fixture, None);
        let mut lookups = DafLookups {
            projects: &mut m.projects,
            organizations: &mut m.organizations,
            customers: &mut m.customers,
            does: &mut m.does,
        };
        let mut products = products;
        let mut suppliers = suppliers;
        DafService::new(repository)
            .create(command, &mut lookups, &mut products, &mut suppliers)
            .await
    }

    #[tokio::test]
    async fn the_dossier_is_prefilled_and_not_saved() {
        let fixture = fixture();
        let mut m = mocks(&fixture, None);
        let mut lookups = DafLookups {
            projects: &mut m.projects,
            organizations: &mut m.organizations,
            customers: &mut m.customers,
            does: &mut m.does,
        };

        let dossier = DafService::new(repository(&fixture, false, vec![]))
            .dossier(fixture.organization_id, fixture.project.id, &mut lookups)
            .await
            .unwrap();

        assert_eq!(dossier.title, "Parc Montgroux");
        assert_eq!(dossier.issuer, "Treyve Paysages");
        assert_eq!(dossier.project_owner.as_deref(), Some("Commune de Cébazat"));
        assert_eq!(dossier.project_manager, None);
        assert_eq!(dossier.updated_at, None);
    }

    #[tokio::test]
    async fn the_prefilled_manager_comes_from_the_doe() {
        let fixture = fixture();
        let now = Utc::now();
        let doe = Doe {
            id: DoeId(Uuid::new_v4()),
            organization_id: fixture.organization_id,
            project_id: fixture.project.id,
            model_id: None,
            title: "DOE".to_owned(),
            place: None,
            lot: None,
            issued_on: now.date_naive(),
            author: None,
            revision: Revision::try_from("A".to_owned()).unwrap(),
            cover_photo_key: None,
            parties: vec![DoeParty {
                role: PartyRole::ProjectManager,
                name: "Atelier Casa".to_owned(),
                address: None,
                phone: None,
                email: None,
                logo_key: None,
                contact_name: None,
                contact_role: None,
                contact_email: None,
                contact_phone: None,
            }],
            sections: Vec::new(),
            created_at: now,
            updated_at: now,
        };
        let mut m = mocks(&fixture, Some(doe));
        let mut lookups = DafLookups {
            projects: &mut m.projects,
            organizations: &mut m.organizations,
            customers: &mut m.customers,
            does: &mut m.does,
        };

        let dossier = DafService::new(repository(&fixture, false, vec![]))
            .dossier(fixture.organization_id, fixture.project.id, &mut lookups)
            .await
            .unwrap();

        assert_eq!(dossier.project_manager.as_deref(), Some("Atelier Casa"));
    }

    #[tokio::test]
    async fn the_stored_dossier_wins() {
        let fixture = fixture();
        let mut m = mocks(&fixture, None);
        let mut lookups = DafLookups {
            projects: &mut m.projects,
            organizations: &mut m.organizations,
            customers: &mut m.customers,
            does: &mut m.does,
        };

        let dossier = DafService::new(repository(&fixture, true, vec![]))
            .dossier(fixture.organization_id, fixture.project.id, &mut lookups)
            .await
            .unwrap();

        assert_eq!(dossier.title, "Stocké");
    }

    #[tokio::test]
    async fn a_project_of_another_organization_is_not_found() {
        let fixture = fixture();
        let mut m = mocks(&fixture, None);
        let mut lookups = DafLookups {
            projects: &mut m.projects,
            organizations: &mut m.organizations,
            customers: &mut m.customers,
            does: &mut m.does,
        };

        let error = DafService::new(repository(&fixture, false, vec![]))
            .dossier(
                OrganizationId(Uuid::new_v4()),
                fixture.project.id,
                &mut lookups,
            )
            .await
            .unwrap_err();

        assert!(matches!(error, CoreError::NotFound));
    }

    #[tokio::test]
    async fn saving_the_dossier_trims_and_clears_blank_options() {
        let fixture = fixture();
        let mut m = mocks(&fixture, None);
        let mut lookups = DafLookups {
            projects: &mut m.projects,
            organizations: &mut m.organizations,
            customers: &mut m.customers,
            does: &mut m.does,
        };

        let saved = DafService::new(repository(&fixture, false, vec![]))
            .save_dossier(
                UpdateDafDossierCommand {
                    actor: actor(),
                    organization_id: fixture.organization_id,
                    project_id: fixture.project.id,
                    title: "  Parc  ".to_owned(),
                    issuer: " Treyve ".to_owned(),
                    project_owner: Some("   ".to_owned()),
                    project_manager: Some(" Casa ".to_owned()),
                },
                &mut lookups,
            )
            .await
            .unwrap();

        assert_eq!(saved.title, "Parc");
        assert_eq!(saved.issuer, "Treyve");
        assert_eq!(saved.project_owner, None);
        assert_eq!(saved.project_manager.as_deref(), Some("Casa"));
    }

    #[tokio::test]
    async fn a_blank_dossier_title_is_a_conflict() {
        let fixture = fixture();
        let mut m = mocks(&fixture, None);
        let mut lookups = DafLookups {
            projects: &mut m.projects,
            organizations: &mut m.organizations,
            customers: &mut m.customers,
            does: &mut m.does,
        };

        let error = DafService::new(repository(&fixture, false, vec![]))
            .save_dossier(
                UpdateDafDossierCommand {
                    actor: actor(),
                    organization_id: fixture.organization_id,
                    project_id: fixture.project.id,
                    title: " ".to_owned(),
                    issuer: "Treyve".to_owned(),
                    project_owner: None,
                    project_manager: None,
                },
                &mut lookups,
            )
            .await
            .unwrap_err();

        assert!(matches!(error, CoreError::Conflict(_)));
    }

    #[tokio::test]
    async fn the_first_daf_is_number_one() {
        let fixture = fixture();
        let command = create_command(&fixture, Some("Ganivelle"));

        let daf = create(
            &fixture,
            repository(&fixture, false, vec![]),
            command,
            MockProductRepository::new(),
            MockSupplierRepository::new(),
        )
        .await
        .unwrap();

        assert_eq!(daf.number.get(), 1);
    }

    #[tokio::test]
    async fn the_next_number_follows_the_highest() {
        let fixture = fixture();
        let command = create_command(&fixture, Some(" Ganivelle "));

        let daf = create(
            &fixture,
            repository(&fixture, false, vec![1, 2]),
            command,
            MockProductRepository::new(),
            MockSupplierRepository::new(),
        )
        .await
        .unwrap();

        assert_eq!(daf.number.get(), 3);
        assert_eq!(daf.designation.as_str(), "Ganivelle");
        assert_eq!(daf.status, DafStatus::Pending);
        assert_eq!(daf.issued_on, Utc::now().date_naive());
        assert_eq!(
            daf.installer.as_deref(),
            Some("Treyve Paysages\n3 chemin des Vignes\n03120 Lapalisse")
        );
        assert_eq!(daf.project_owner, None);
    }

    #[tokio::test]
    async fn a_gap_does_not_reuse_a_number() {
        let fixture = fixture();
        let command = create_command(&fixture, Some("Ganivelle"));

        let daf = create(
            &fixture,
            repository(&fixture, true, vec![1, 5]),
            command,
            MockProductRepository::new(),
            MockSupplierRepository::new(),
        )
        .await
        .unwrap();

        assert_eq!(daf.number.get(), 6);
    }

    #[tokio::test]
    async fn the_dossier_is_persisted_with_the_first_daf() {
        let fixture = fixture();
        let mut repository = MockDafRepository::new();
        repository
            .expect_find_dossier()
            .returning(|_| Box::pin(async { Ok(None) }));
        repository
            .expect_list_summaries()
            .returning(|_| Box::pin(async { Ok(Vec::new()) }));
        repository
            .expect_upsert_dossier()
            .times(1)
            .returning(|dossier| {
                let dossier = dossier.clone();
                Box::pin(async move { Ok(dossier) })
            });
        repository.expect_insert().returning(|daf| {
            let daf = daf.clone();
            Box::pin(async move { Ok(daf) })
        });

        create(
            &fixture,
            repository,
            create_command(&fixture, Some("Ganivelle")),
            MockProductRepository::new(),
            MockSupplierRepository::new(),
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn a_stored_dossier_is_not_rewritten() {
        let fixture = fixture();
        let mut repository = MockDafRepository::new();
        let stored = DafDossier {
            organization_id: fixture.organization_id,
            project_id: fixture.project.id,
            title: "Stocké".to_owned(),
            issuer: "Moi".to_owned(),
            project_owner: None,
            project_manager: None,
            updated_at: Some(Utc::now()),
        };
        repository.expect_find_dossier().returning(move |_| {
            let stored = stored.clone();
            Box::pin(async move { Ok(Some(stored)) })
        });
        repository
            .expect_list_summaries()
            .returning(|_| Box::pin(async { Ok(Vec::new()) }));
        repository.expect_upsert_dossier().never();
        repository.expect_insert().returning(|daf| {
            let daf = daf.clone();
            Box::pin(async move { Ok(daf) })
        });

        create(
            &fixture,
            repository,
            create_command(&fixture, Some("Ganivelle")),
            MockProductRepository::new(),
            MockSupplierRepository::new(),
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn number_999_has_no_successor() {
        let fixture = fixture();

        let error = create(
            &fixture,
            repository(&fixture, true, vec![999]),
            create_command(&fixture, Some("Ganivelle")),
            MockProductRepository::new(),
            MockSupplierRepository::new(),
        )
        .await
        .unwrap_err();

        assert!(matches!(error, CoreError::Conflict(_)));
    }

    #[tokio::test]
    async fn a_project_holds_at_most_two_hundred_dafs() {
        let fixture = fixture();

        let error = create(
            &fixture,
            repository(&fixture, true, (1..=200).collect()),
            create_command(&fixture, Some("Ganivelle")),
            MockProductRepository::new(),
            MockSupplierRepository::new(),
        )
        .await
        .unwrap_err();

        assert!(matches!(error, CoreError::Conflict(_)));
    }

    #[tokio::test]
    async fn a_daf_needs_a_designation_or_a_product() {
        let fixture = fixture();

        let error = create(
            &fixture,
            repository(&fixture, true, vec![]),
            create_command(&fixture, None),
            MockProductRepository::new(),
            MockSupplierRepository::new(),
        )
        .await
        .unwrap_err();

        assert!(matches!(error, CoreError::Conflict(_)));
    }

    fn product(organization_id: OrganizationId, supplier_id: Option<SupplierId>) -> Product {
        let now = Utc::now();
        Product {
            id: ProductId(Uuid::new_v4()),
            organization_id,
            name: "Ensemble table bois".to_owned(),
            sku: None,
            unit: "u".to_owned(),
            unit_price_cents: 1000,
            default_vat_rate_bp: None,
            description: None,
            photo_keys: vec!["photo-1".to_owned(), "photo-2".to_owned()],
            supplier_id,
            manufacturer: None,
            technical_specs: Some("Bois lamellé".to_owned()),
            datasheet_keys: vec!["sheet-1".to_owned(), "sheet-2".to_owned()],
            catalog: Default::default(),
            usage: Default::default(),
            deleted_at: None,
            created_at: now,
            updated_at: now,
        }
    }

    fn supplier(organization_id: OrganizationId) -> Supplier {
        let now = Utc::now();
        Supplier {
            id: SupplierId(Uuid::new_v4()),
            organization_id,
            name: "KOMPAN".to_owned(),
            registration_number: None,
            vat_number: None,
            address: Some("1 rue du Parc".to_owned()),
            phone: Some("0102030405".to_owned()),
            email: None,
            profile: Default::default(),
            deleted_at: None,
            created_at: now,
            updated_at: now,
        }
    }

    fn products_returning(product: Product) -> MockProductRepository {
        let mut repository = MockProductRepository::new();
        repository.expect_find_by_id().returning(move |_| {
            let product = product.clone();
            Box::pin(async move { Ok(Some(product)) })
        });
        repository
    }

    fn suppliers_returning(supplier: Supplier) -> MockSupplierRepository {
        let mut repository = MockSupplierRepository::new();
        repository.expect_find_by_id().returning(move |_| {
            let supplier = supplier.clone();
            Box::pin(async move { Ok(Some(supplier)) })
        });
        repository
    }

    #[tokio::test]
    async fn a_product_is_copied_into_the_daf() {
        let fixture = fixture();
        let supplier = supplier(fixture.organization_id);
        let product = product(fixture.organization_id, Some(supplier.id));
        let mut command = create_command(&fixture, None);
        command.product_id = Some(product.id);
        let product_id = product.id;

        let daf = create(
            &fixture,
            repository(&fixture, true, vec![]),
            command,
            products_returning(product),
            suppliers_returning(supplier),
        )
        .await
        .unwrap();

        assert_eq!(daf.designation.as_str(), "Ensemble table bois");
        assert_eq!(daf.source_product_id, Some(product_id));
        assert_eq!(daf.specs.as_deref(), Some("Bois lamellé"));
        assert_eq!(daf.photo_key.as_deref(), Some("photo-1"));
        let snapshot = daf.supplier.unwrap();
        assert_eq!(snapshot.name, "KOMPAN");
        assert_eq!(snapshot.address.as_deref(), Some("1 rue du Parc"));
        assert_eq!(snapshot.phone.as_deref(), Some("0102030405"));
        assert_eq!(
            daf.attachments,
            vec![
                DafAttachment {
                    key: "sheet-1".to_owned(),
                    name: "Fiche technique".to_owned()
                },
                DafAttachment {
                    key: "sheet-2".to_owned(),
                    name: "Fiche technique 2".to_owned()
                },
            ]
        );
    }

    #[tokio::test]
    async fn a_given_designation_beats_the_product_name() {
        let fixture = fixture();
        let product = product(fixture.organization_id, None);
        let mut command = create_command(&fixture, Some("Table pique-nique"));
        command.product_id = Some(product.id);

        let daf = create(
            &fixture,
            repository(&fixture, true, vec![]),
            command,
            products_returning(product),
            MockSupplierRepository::new(),
        )
        .await
        .unwrap();

        assert_eq!(daf.designation.as_str(), "Table pique-nique");
        assert_eq!(daf.supplier, None);
    }

    #[tokio::test]
    async fn a_product_of_another_organization_is_not_found() {
        let fixture = fixture();
        let product = product(OrganizationId(Uuid::new_v4()), None);
        let mut command = create_command(&fixture, None);
        command.product_id = Some(product.id);

        let error = create(
            &fixture,
            repository(&fixture, true, vec![]),
            command,
            products_returning(product),
            MockSupplierRepository::new(),
        )
        .await
        .unwrap_err();

        assert!(matches!(error, CoreError::NotFound));
    }

    fn existing_daf(fixture: &Fixture) -> Daf {
        let now = Utc::now();
        Daf {
            id: DafId(Uuid::new_v4()),
            organization_id: fixture.organization_id,
            project_id: fixture.project.id,
            number: DafNumber::try_from(3).unwrap(),
            designation: DafDesignation::try_from("Ganivelle".to_owned()).unwrap(),
            price_ref: None,
            source_product_id: None,
            supplier: None,
            installer: None,
            specs: None,
            observations: None,
            photo_key: None,
            attachments: Vec::new(),
            project_owner: None,
            project_manager: None,
            status: DafStatus::Pending,
            sent_on: None,
            issued_on: now.date_naive(),
            returned_on: None,
            review_note: None,
            created_at: now,
            updated_at: now,
        }
    }

    fn update_command(daf: &Daf) -> UpdateDafCommand {
        UpdateDafCommand {
            actor: actor(),
            id: daf.id,
            number: daf.number,
            designation: daf.designation.clone(),
            price_ref: None,
            supplier: None,
            installer: None,
            specs: None,
            observations: None,
            photo_key: None,
            attachments: Vec::new(),
            project_owner: None,
            project_manager: None,
            status: DafStatus::Pending,
            sent_on: None,
            issued_on: daf.issued_on,
            returned_on: None,
            review_note: None,
            expected_updated_at: None,
        }
    }

    async fn replace_with(command: UpdateDafCommand, daf: Daf) -> Result<Daf, CoreError> {
        let mut repository = MockDafRepository::new();
        repository.expect_find_by_id().returning(move |_| {
            let daf = daf.clone();
            Box::pin(async move { Ok(Some(daf)) })
        });
        repository.expect_update().returning(|daf| {
            let daf = daf.clone();
            Box::pin(async move { Ok(daf) })
        });
        DafService::new(repository).replace(command).await
    }

    #[tokio::test]
    async fn replace_refuses_a_stale_version() {
        let fixture = fixture();
        let daf = existing_daf(&fixture);
        let mut command = update_command(&daf);
        command.expected_updated_at = Some(daf.updated_at - chrono::Duration::seconds(5));

        let error = replace_with(command, daf).await.unwrap_err();

        assert!(matches!(error, CoreError::Conflict(_)));
    }

    #[tokio::test]
    async fn replace_accepts_the_current_version() {
        let fixture = fixture();
        let daf = existing_daf(&fixture);
        let mut command = update_command(&daf);
        command.expected_updated_at = Some(daf.updated_at);

        replace_with(command, daf).await.unwrap();
    }

    #[tokio::test]
    async fn replace_refuses_more_than_twenty_attachments() {
        let fixture = fixture();
        let daf = existing_daf(&fixture);
        let mut command = update_command(&daf);
        command.attachments = (0..21)
            .map(|index| DafAttachment {
                key: format!("k{index}"),
                name: format!("n{index}"),
            })
            .collect();

        let error = replace_with(command, daf).await.unwrap_err();

        assert!(matches!(error, CoreError::Conflict(_)));
    }

    #[tokio::test]
    async fn replace_turns_blank_strings_into_none_and_keeps_identity() {
        let fixture = fixture();
        let daf = existing_daf(&fixture);
        let mut command = update_command(&daf);
        command.price_ref = Some("  ".to_owned());
        command.installer = Some(" Treyve ".to_owned());
        command.specs = Some("".to_owned());
        command.photo_key = Some(" ".to_owned());
        command.project_owner = Some("  ".to_owned());
        command.review_note = Some(" ok ".to_owned());
        command.status = DafStatus::Accepted;
        command.sent_on = NaiveDate::from_ymd_opt(2026, 11, 2);

        let saved = replace_with(command, daf.clone()).await.unwrap();

        assert_eq!(saved.id, daf.id);
        assert_eq!(saved.organization_id, daf.organization_id);
        assert_eq!(saved.project_id, daf.project_id);
        assert_eq!(saved.created_at, daf.created_at);
        assert_eq!(saved.price_ref, None);
        assert_eq!(saved.installer.as_deref(), Some("Treyve"));
        assert_eq!(saved.specs, None);
        assert_eq!(saved.photo_key, None);
        assert_eq!(saved.project_owner, None);
        assert_eq!(saved.review_note.as_deref(), Some("ok"));
        assert_eq!(saved.status, DafStatus::Accepted);
        assert!(saved.updated_at >= daf.updated_at);
    }

    #[tokio::test]
    async fn replace_surfaces_a_taken_number() {
        let fixture = fixture();
        let daf = existing_daf(&fixture);
        let command = update_command(&daf);
        let mut repository = MockDafRepository::new();
        let stored = daf.clone();
        repository.expect_find_by_id().returning(move |_| {
            let stored = stored.clone();
            Box::pin(async move { Ok(Some(stored)) })
        });
        repository
            .expect_update()
            .returning(|_| Box::pin(async { Err(CoreError::Conflict("number taken".to_owned())) }));

        let error = DafService::new(repository)
            .replace(command)
            .await
            .unwrap_err();

        assert!(matches!(error, CoreError::Conflict(_)));
    }

    #[tokio::test]
    async fn a_daf_of_another_organization_is_not_found() {
        let fixture = fixture();
        let daf = existing_daf(&fixture);
        let id = daf.id;
        let mut repository = MockDafRepository::new();
        repository.expect_find_by_id().returning(move |_| {
            let daf = daf.clone();
            Box::pin(async move { Ok(Some(daf)) })
        });

        let error = DafService::new(repository)
            .get_in(OrganizationId(Uuid::new_v4()), id)
            .await
            .unwrap_err();

        assert!(matches!(error, CoreError::NotFound));
    }
}
