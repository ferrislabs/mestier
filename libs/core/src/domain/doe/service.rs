use chrono::Utc;
use common::{CoreError, generate_uuid_v7};

use crate::{
    Customer, Organization, Project, ProjectId,
    domain::{
        customer::ports::CustomerRepository,
        customer_context::ports::CustomerContextRepository,
        doe::{
            Doe, DoeFile, DoeId, DoeParty, DoeSection, DoeSectionContent, DoeSupply, PartyRole,
            Revision, SupplierSnapshot,
            commands::{CreateDoeCommand, DraftSupplyFromProductCommand, UpdateDoeCommand},
            ports::DoeRepository,
            render::DoeDocument,
        },
        doe_model::{
            DoeModel, DoeModelSection, DoeModelSectionKind, DoeSettings,
            ports::{DoeModelRepository, DoeSettingsRepository},
        },
        organization::ports::OrganizationRepository,
        product::ports::ProductRepository,
        project::ports::ProjectRepository,
        supplier_invoice::ports::SupplierRepository,
    },
};

pub const MAX_SECTIONS: usize = 30;

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

fn clean_file(file: DoeFile) -> DoeFile {
    DoeFile {
        key: file.key,
        caption: blank_to_none(file.caption),
    }
}

fn clean_supply(supply: DoeSupply) -> Result<DoeSupply, CoreError> {
    Ok(DoeSupply {
        id: supply.id,
        source_product_id: supply.source_product_id,
        title: blank_to_none(supply.title),
        designation: required(&supply.designation, "a supply designation")?,
        dpgf_ref: blank_to_none(supply.dpgf_ref),
        destination: blank_to_none(supply.destination),
        supplier: match supply.supplier {
            Some(supplier) => Some(SupplierSnapshot {
                name: required(&supplier.name, "a supplier name")?,
                address: blank_to_none(supplier.address),
                phone: blank_to_none(supplier.phone),
                email: blank_to_none(supplier.email),
            }),
            None => None,
        },
        maker: blank_to_none(supply.maker),
        specs: blank_to_none(supply.specs),
        photos: supply.photos.into_iter().map(clean_file).collect(),
        datasheet_keys: supply.datasheet_keys,
    })
}

fn clean_content(content: DoeSectionContent) -> Result<DoeSectionContent, CoreError> {
    Ok(match content {
        DoeSectionContent::Supplies {
            maker_label,
            supplies,
        } => DoeSectionContent::Supplies {
            maker_label,
            supplies: supplies
                .into_iter()
                .map(clean_supply)
                .collect::<Result<_, _>>()?,
        },
        DoeSectionContent::Photos { per_page, photos } => DoeSectionContent::Photos {
            per_page,
            photos: photos.into_iter().map(clean_file).collect(),
        },
        DoeSectionContent::Plans { plans } => DoeSectionContent::Plans {
            plans: plans.into_iter().map(clean_file).collect(),
        },
        blank @ DoeSectionContent::BlankPages { .. } => blank,
    })
}

fn clean_section(section: DoeSection) -> Result<DoeSection, CoreError> {
    Ok(DoeSection {
        id: section.id,
        title: required(&section.title, "a section title")?,
        enabled: section.enabled,
        content: clean_content(section.content)?,
    })
}

fn clean_party(party: DoeParty) -> Result<DoeParty, CoreError> {
    Ok(DoeParty {
        role: party.role,
        name: required(&party.name, "a party name")?,
        address: blank_to_none(party.address),
        phone: blank_to_none(party.phone),
        email: blank_to_none(party.email),
        logo_key: blank_to_none(party.logo_key),
    })
}

fn section_from_model(section: &DoeModelSection) -> DoeSection {
    let content = match &section.kind {
        DoeModelSectionKind::Supplies { maker_label } => DoeSectionContent::Supplies {
            maker_label: *maker_label,
            supplies: Vec::new(),
        },
        DoeModelSectionKind::Photos { per_page } => DoeSectionContent::Photos {
            per_page: *per_page,
            photos: Vec::new(),
        },
        DoeModelSectionKind::Plans => DoeSectionContent::Plans { plans: Vec::new() },
        DoeModelSectionKind::BlankPages { count } => {
            DoeSectionContent::BlankPages { count: *count }
        }
    };
    DoeSection {
        id: generate_uuid_v7(),
        title: section.title.clone(),
        enabled: section.enabled,
        content,
    }
}

fn join_lines(lines: impl IntoIterator<Item = Option<String>>) -> Option<String> {
    let joined = lines
        .into_iter()
        .filter_map(blank_to_none)
        .collect::<Vec<_>>()
        .join("\n");
    (!joined.is_empty()).then_some(joined)
}

fn contractor_party(organization: &Organization) -> DoeParty {
    let locality = join_lines([
        organization.address_postal_code.clone(),
        organization.address_city.clone(),
    ])
    .map(|text| text.replace('\n', " "));
    DoeParty {
        role: PartyRole::Contractor,
        name: organization.name.clone(),
        address: join_lines([
            organization.address_line1.clone(),
            organization.address_line2.clone(),
            locality,
        ]),
        phone: blank_to_none(organization.contact_phone.clone()),
        email: blank_to_none(organization.contact_email.clone()),
        logo_key: None,
    }
}

fn owner_party(customer: &Customer, address: Option<String>) -> DoeParty {
    DoeParty {
        role: PartyRole::ProjectOwner,
        name: customer.name.clone(),
        address,
        phone: blank_to_none(customer.phone.clone()),
        email: blank_to_none(customer.email.clone()),
        logo_key: None,
    }
}

pub struct DoeService<DR>
where
    DR: DoeRepository,
{
    doe_repository: DR,
}

impl<DR> DoeService<DR>
where
    DR: DoeRepository,
{
    pub fn new(doe_repository: DR) -> Self {
        Self { doe_repository }
    }

    pub async fn get_doe(&mut self, id: DoeId) -> Result<Doe, CoreError> {
        self.doe_repository
            .find_by_id(id)
            .await?
            .ok_or(CoreError::NotFound)
    }

    pub async fn get_project_doe(&mut self, project_id: ProjectId) -> Result<Doe, CoreError> {
        self.doe_repository
            .find_by_project(project_id)
            .await?
            .ok_or(CoreError::NotFound)
    }

    pub async fn create_from_model<MR, PR, CR, CCR, OR>(
        &mut self,
        command: CreateDoeCommand,
        models: &mut MR,
        projects: &mut PR,
        customers: &mut CR,
        contexts: &mut CCR,
        organizations: &mut OR,
    ) -> Result<Doe, CoreError>
    where
        MR: DoeModelRepository,
        PR: ProjectRepository,
        CR: CustomerRepository,
        CCR: CustomerContextRepository,
        OR: OrganizationRepository,
    {
        let project: Project = projects
            .find_by_id(command.project_id)
            .await?
            .filter(|project| project.organization_id == command.organization_id)
            .ok_or(CoreError::NotFound)?;

        let model: DoeModel = models
            .find_by_id(command.model_id)
            .await?
            .filter(|model| model.organization_id == command.organization_id)
            .ok_or(CoreError::NotFound)?;
        if model.is_archived() {
            return Err(CoreError::Conflict("the DOE model is archived".to_owned()));
        }

        if self
            .doe_repository
            .find_by_project(project.id)
            .await?
            .is_some()
        {
            return Err(CoreError::Conflict(
                "the project already has a DOE".to_owned(),
            ));
        }

        let mut parties = Vec::new();
        if let Some(customer_id) = project.customer_id
            && let Some(customer) = customers
                .find_by_id(customer_id)
                .await?
                .filter(|customer| customer.organization_id == command.organization_id)
        {
            let address = match project.customer_context_id {
                Some(context_id) => contexts
                    .find_by_id(context_id)
                    .await?
                    .filter(|context| context.customer_id == customer.id)
                    .and_then(|context| context.address_line),
                None => None,
            };
            parties.push(owner_party(&customer, blank_to_none(address)));
        }
        let organization = organizations
            .find_by_id(command.organization_id)
            .await?
            .ok_or(CoreError::NotFound)?;
        parties.push(contractor_party(&organization));

        let now = Utc::now();
        let doe = Doe {
            id: DoeId(generate_uuid_v7()),
            organization_id: command.organization_id,
            project_id: project.id,
            title: project.name.clone(),
            place: None,
            lot: None,
            issued_on: now.date_naive(),
            author: None,
            revision: Revision::try_from("A".to_owned())?,
            parties,
            sections: model.sections.iter().map(section_from_model).collect(),
            created_at: now,
            updated_at: now,
        };

        self.doe_repository.insert(&doe).await
    }

    pub async fn replace(&mut self, command: UpdateDoeCommand) -> Result<Doe, CoreError> {
        let existing = self.get_doe(command.id).await?;

        if command.sections.len() > MAX_SECTIONS {
            return Err(CoreError::Conflict(format!(
                "a DOE holds at most {MAX_SECTIONS} sections"
            )));
        }

        let doe = Doe {
            title: required(&command.title, "the DOE title")?,
            place: blank_to_none(command.place),
            lot: blank_to_none(command.lot),
            issued_on: command.issued_on,
            author: blank_to_none(command.author),
            revision: command.revision,
            parties: command
                .parties
                .into_iter()
                .map(clean_party)
                .collect::<Result<_, _>>()?,
            sections: command
                .sections
                .into_iter()
                .map(clean_section)
                .collect::<Result<_, _>>()?,
            updated_at: Utc::now(),
            ..existing
        };

        self.doe_repository.update(&doe).await
    }

    pub async fn draft_supply_from_product<PR, SR>(
        &mut self,
        command: DraftSupplyFromProductCommand,
        products: &mut PR,
        suppliers: &mut SR,
    ) -> Result<DoeSupply, CoreError>
    where
        PR: ProductRepository,
        SR: SupplierRepository,
    {
        let doe = self.get_doe(command.doe_id).await?;

        let product = products
            .find_by_id(command.product_id)
            .await?
            .filter(|product| {
                product.organization_id == doe.organization_id && product.deleted_at.is_none()
            })
            .ok_or(CoreError::NotFound)?;

        let supplier = match product.supplier_id {
            Some(supplier_id) => suppliers
                .find_by_id(supplier_id)
                .await?
                .filter(|supplier| supplier.organization_id == doe.organization_id),
            None => None,
        };

        Ok(DoeSupply {
            id: generate_uuid_v7(),
            source_product_id: Some(product.id),
            title: None,
            designation: product.name,
            dpgf_ref: None,
            destination: None,
            supplier: supplier.map(|supplier| SupplierSnapshot {
                name: supplier.name,
                address: supplier.address,
                phone: supplier.phone,
                email: supplier.email,
            }),
            maker: product.manufacturer,
            specs: product.technical_specs,
            photos: product
                .photo_keys
                .into_iter()
                .map(|key| DoeFile { key, caption: None })
                .collect(),
            datasheet_keys: product.datasheet_keys,
        })
    }

    pub async fn document<SR>(
        &mut self,
        id: DoeId,
        settings: &mut SR,
    ) -> Result<DoeDocument, CoreError>
    where
        SR: DoeSettingsRepository,
    {
        let doe = self.get_doe(id).await?;
        let settings = settings
            .find(doe.organization_id)
            .await?
            .unwrap_or_else(|| DoeSettings::default_for(doe.organization_id));
        Ok(DoeDocument::from_doe(&doe, &settings))
    }
}

#[cfg(test)]
mod tests {
    use chrono::NaiveDate;
    use uuid::Uuid;

    use super::*;
    use crate::{
        CustomerContext, CustomerContextId, CustomerId, DoeModelId, OrganizationId, Product,
        ProductId, Supplier, SupplierId,
        domain::{
            customer::ports::MockCustomerRepository,
            customer_context::ports::MockCustomerContextRepository,
            doe::{BlankPageCount, MakerLabel, PhotosPerPage, ports::MockDoeRepository},
            doe_model::ports::{MockDoeModelRepository, MockDoeSettingsRepository},
            organization::ports::MockOrganizationRepository,
            product::ports::MockProductRepository,
            project::ports::MockProjectRepository,
            supplier_invoice::ports::MockSupplierRepository,
        },
    };

    fn actor() -> authz::Subject {
        authz::Subject::system()
    }

    fn existing_doe(organization_id: OrganizationId) -> Doe {
        let now = Utc::now();
        Doe {
            id: DoeId(Uuid::new_v4()),
            organization_id,
            project_id: ProjectId(Uuid::new_v4()),
            title: "Avenue".to_owned(),
            place: None,
            lot: None,
            issued_on: NaiveDate::from_ymd_opt(2026, 10, 7).unwrap(),
            author: None,
            revision: Revision::try_from("A".to_owned()).unwrap(),
            parties: Vec::new(),
            sections: Vec::new(),
            created_at: now,
            updated_at: now,
        }
    }

    fn doe_repository_returning(doe: Doe) -> MockDoeRepository {
        let mut repository = MockDoeRepository::new();
        repository.expect_find_by_id().returning(move |_| {
            let doe = doe.clone();
            Box::pin(async move { Ok(Some(doe)) })
        });
        repository.expect_update().returning(|doe| {
            let doe = doe.clone();
            Box::pin(async move { Ok(doe) })
        });
        repository
    }

    fn update_command(doe: &Doe) -> UpdateDoeCommand {
        UpdateDoeCommand {
            actor: actor(),
            id: doe.id,
            title: "Avenue du 8 mai".to_owned(),
            place: None,
            lot: None,
            issued_on: doe.issued_on,
            author: None,
            revision: doe.revision.clone(),
            parties: Vec::new(),
            sections: Vec::new(),
        }
    }

    fn supply(designation: &str) -> DoeSupply {
        DoeSupply {
            id: Uuid::new_v4(),
            source_product_id: None,
            title: None,
            designation: designation.to_owned(),
            dpgf_ref: None,
            destination: None,
            supplier: None,
            maker: None,
            specs: None,
            photos: Vec::new(),
            datasheet_keys: Vec::new(),
        }
    }

    fn section(title: &str, content: DoeSectionContent) -> DoeSection {
        DoeSection {
            id: Uuid::new_v4(),
            title: title.to_owned(),
            enabled: true,
            content,
        }
    }

    fn supplies_section(title: &str, supplies: Vec<DoeSupply>) -> DoeSection {
        section(
            title,
            DoeSectionContent::Supplies {
                maker_label: MakerLabel::Manufacturer,
                supplies,
            },
        )
    }

    async fn replace_with(command: UpdateDoeCommand, doe: Doe) -> Result<Doe, CoreError> {
        DoeService::new(doe_repository_returning(doe))
            .replace(command)
            .await
    }

    #[tokio::test]
    async fn replace_rejects_a_blank_title() {
        let doe = existing_doe(OrganizationId(Uuid::new_v4()));
        let mut command = update_command(&doe);
        command.title = "   ".to_owned();

        let err = replace_with(command, doe).await.unwrap_err();

        assert!(matches!(err, CoreError::Conflict(_)));
    }

    #[tokio::test]
    async fn replace_rejects_more_than_thirty_sections() {
        let doe = existing_doe(OrganizationId(Uuid::new_v4()));
        let mut command = update_command(&doe);
        command.sections = (0..31)
            .map(|index| {
                section(
                    &format!("Thème {index}"),
                    DoeSectionContent::Plans { plans: Vec::new() },
                )
            })
            .collect();

        let err = replace_with(command, doe).await.unwrap_err();

        assert!(matches!(err, CoreError::Conflict(_)));
    }

    #[tokio::test]
    async fn replace_rejects_a_blank_section_title() {
        let doe = existing_doe(OrganizationId(Uuid::new_v4()));
        let mut command = update_command(&doe);
        command.sections = vec![supplies_section(" ", vec![])];

        let err = replace_with(command, doe).await.unwrap_err();

        assert!(matches!(err, CoreError::Conflict(_)));
    }

    #[tokio::test]
    async fn replace_rejects_a_blank_designation() {
        let doe = existing_doe(OrganizationId(Uuid::new_v4()));
        let mut command = update_command(&doe);
        command.sections = vec![supplies_section("Végétaux", vec![supply("  ")])];

        let err = replace_with(command, doe).await.unwrap_err();

        assert!(matches!(err, CoreError::Conflict(_)));
    }

    #[tokio::test]
    async fn replace_rejects_a_blank_party_name() {
        let doe = existing_doe(OrganizationId(Uuid::new_v4()));
        let mut command = update_command(&doe);
        command.parties = vec![DoeParty {
            role: PartyRole::ProjectManager,
            name: String::new(),
            address: None,
            phone: None,
            email: None,
            logo_key: None,
        }];

        let err = replace_with(command, doe).await.unwrap_err();

        assert!(matches!(err, CoreError::Conflict(_)));
    }

    #[tokio::test]
    async fn replace_trims_text_and_keeps_identity() {
        let doe = existing_doe(OrganizationId(Uuid::new_v4()));
        let mut command = update_command(&doe);
        command.title = "  Avenue du 8 mai ".to_owned();
        command.place = Some("   ".to_owned());
        command.author = Some(" Erwan ".to_owned());
        let mut item = supply(" Arbres ");
        item.specs = Some("  ".to_owned());
        command.sections = vec![
            supplies_section("Végétaux", vec![item]),
            section(
                "Photos",
                DoeSectionContent::Photos {
                    per_page: PhotosPerPage::Two,
                    photos: vec![DoeFile {
                        key: "k".to_owned(),
                        caption: Some("  ".to_owned()),
                    }],
                },
            ),
            section(
                "Libre",
                DoeSectionContent::BlankPages {
                    count: BlankPageCount::try_from(2).unwrap(),
                },
            ),
        ];

        let saved = replace_with(command, doe.clone()).await.unwrap();

        assert_eq!(saved.id, doe.id);
        assert_eq!(saved.organization_id, doe.organization_id);
        assert_eq!(saved.project_id, doe.project_id);
        assert_eq!(saved.created_at, doe.created_at);
        assert_eq!(saved.title, "Avenue du 8 mai");
        assert_eq!(saved.place, None);
        assert_eq!(saved.author.as_deref(), Some("Erwan"));
        let DoeSectionContent::Supplies { supplies, .. } = &saved.sections[0].content else {
            panic!("expected supplies");
        };
        assert_eq!(supplies[0].designation, "Arbres");
        assert_eq!(supplies[0].specs, None);
        let DoeSectionContent::Photos { photos, .. } = &saved.sections[1].content else {
            panic!("expected photos");
        };
        assert_eq!(photos[0].caption, None);
    }

    fn product(organization_id: OrganizationId, supplier_id: Option<SupplierId>) -> Product {
        let now = Utc::now();
        Product {
            id: ProductId(Uuid::new_v4()),
            organization_id,
            name: "Banc Garden".to_owned(),
            sku: None,
            unit: "u".to_owned(),
            unit_price_cents: 1000,
            default_vat_rate_bp: None,
            description: None,
            photo_keys: vec!["photo-1".to_owned()],
            supplier_id,
            manufacturer: Some("PLAS ECO".to_owned()),
            technical_specs: Some("Plastique recyclé".to_owned()),
            datasheet_keys: vec!["sheet-1".to_owned(), "sheet-2".to_owned()],
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
            name: "PLAS ECO".to_owned(),
            registration_number: None,
            vat_number: None,
            address: Some("1 rue du Parc".to_owned()),
            phone: Some("0102030405".to_owned()),
            email: None,
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
    async fn draft_copies_the_technical_sheet() {
        let organization_id = OrganizationId(Uuid::new_v4());
        let doe = existing_doe(organization_id);
        let supplier = supplier(organization_id);
        let product = product(organization_id, Some(supplier.id));
        let command = DraftSupplyFromProductCommand {
            actor: actor(),
            doe_id: doe.id,
            product_id: product.id,
        };
        let product_id = product.id;
        let mut service = DoeService::new(doe_repository_returning(doe));

        let draft = service
            .draft_supply_from_product(
                command,
                &mut products_returning(product),
                &mut suppliers_returning(supplier),
            )
            .await
            .unwrap();

        assert_eq!(draft.source_product_id, Some(product_id));
        assert_eq!(draft.designation, "Banc Garden");
        assert_eq!(draft.maker.as_deref(), Some("PLAS ECO"));
        assert_eq!(draft.specs.as_deref(), Some("Plastique recyclé"));
        assert_eq!(draft.datasheet_keys, vec!["sheet-1", "sheet-2"]);
        assert_eq!(draft.photos.len(), 1);
        assert_eq!(draft.photos[0].key, "photo-1");
        assert_eq!(draft.photos[0].caption, None);
        assert_eq!(draft.title, None);
        assert_eq!(draft.dpgf_ref, None);
        assert_eq!(draft.destination, None);
        let snapshot = draft.supplier.unwrap();
        assert_eq!(snapshot.name, "PLAS ECO");
        assert_eq!(snapshot.address.as_deref(), Some("1 rue du Parc"));
        assert_eq!(snapshot.phone.as_deref(), Some("0102030405"));
        assert_eq!(snapshot.email, None);
    }

    #[tokio::test]
    async fn draft_without_supplier_has_no_snapshot() {
        let organization_id = OrganizationId(Uuid::new_v4());
        let doe = existing_doe(organization_id);
        let product = product(organization_id, None);
        let command = DraftSupplyFromProductCommand {
            actor: actor(),
            doe_id: doe.id,
            product_id: product.id,
        };
        let mut service = DoeService::new(doe_repository_returning(doe));

        let draft = service
            .draft_supply_from_product(
                command,
                &mut products_returning(product),
                &mut MockSupplierRepository::new(),
            )
            .await
            .unwrap();

        assert_eq!(draft.supplier, None);
    }

    #[tokio::test]
    async fn draft_refuses_a_product_of_another_organization() {
        let doe = existing_doe(OrganizationId(Uuid::new_v4()));
        let product = product(OrganizationId(Uuid::new_v4()), None);
        let command = DraftSupplyFromProductCommand {
            actor: actor(),
            doe_id: doe.id,
            product_id: product.id,
        };
        let mut service = DoeService::new(doe_repository_returning(doe));

        let err = service
            .draft_supply_from_product(
                command,
                &mut products_returning(product),
                &mut MockSupplierRepository::new(),
            )
            .await
            .unwrap_err();

        assert!(matches!(err, CoreError::NotFound));
    }

    #[tokio::test]
    async fn draft_refuses_a_deleted_product() {
        let organization_id = OrganizationId(Uuid::new_v4());
        let doe = existing_doe(organization_id);
        let mut product = product(organization_id, None);
        product.deleted_at = Some(Utc::now());
        let command = DraftSupplyFromProductCommand {
            actor: actor(),
            doe_id: doe.id,
            product_id: product.id,
        };
        let mut service = DoeService::new(doe_repository_returning(doe));

        let err = service
            .draft_supply_from_product(
                command,
                &mut products_returning(product),
                &mut MockSupplierRepository::new(),
            )
            .await
            .unwrap_err();

        assert!(matches!(err, CoreError::NotFound));
    }

    #[tokio::test]
    async fn document_falls_back_to_default_settings() {
        let doe = existing_doe(OrganizationId(Uuid::new_v4()));
        let id = doe.id;
        let mut settings = MockDoeSettingsRepository::new();
        settings
            .expect_find()
            .returning(|_| Box::pin(async { Ok(None) }));
        let mut service = DoeService::new(doe_repository_returning(doe));

        let document = service.document(id, &mut settings).await.unwrap();

        assert_eq!(document.accent.as_str(), "#0f3d36");
        assert_eq!(document.logo_key, None);
    }

    struct Fixture {
        organization_id: OrganizationId,
        project: Project,
        model: DoeModel,
        customer: Customer,
    }

    fn fixture() -> Fixture {
        let now = Utc::now();
        let organization_id = OrganizationId(Uuid::new_v4());
        let customer = Customer {
            id: CustomerId(Uuid::new_v4()),
            organization_id,
            status: crate::CustomerStatus::Client,
            pipeline_stage: crate::CustomerPipelineStage::New,
            name: "Commune de Lapalisse".to_owned(),
            registration_number: None,
            phone: Some("0400000000".to_owned()),
            email: None,
            deleted_at: None,
            created_at: now,
            updated_at: now,
        };
        let project = Project {
            id: ProjectId(Uuid::new_v4()),
            organization_id,
            name: "Avenue du 8 mai".to_owned(),
            customer_id: Some(customer.id),
            customer_context_id: Some(CustomerContextId(Uuid::new_v4())),
            quote_id: None,
            archived_at: None,
            created_at: now,
            updated_at: now,
        };
        let model = DoeModel {
            id: DoeModelId(Uuid::new_v4()),
            organization_id,
            name: "Paysagiste".to_owned(),
            description: None,
            sections: vec![
                DoeModelSection {
                    title: "Végétaux".to_owned(),
                    enabled: true,
                    kind: DoeModelSectionKind::Supplies {
                        maker_label: MakerLabel::Nursery,
                    },
                },
                DoeModelSection {
                    title: "Jeux".to_owned(),
                    enabled: false,
                    kind: DoeModelSectionKind::Supplies {
                        maker_label: MakerLabel::Manufacturer,
                    },
                },
                DoeModelSection {
                    title: "Photos".to_owned(),
                    enabled: true,
                    kind: DoeModelSectionKind::Photos {
                        per_page: PhotosPerPage::Four,
                    },
                },
            ],
            archived_at: None,
            created_at: now,
            updated_at: now,
        };
        Fixture {
            organization_id,
            project,
            model,
            customer,
        }
    }

    fn organization(id: OrganizationId) -> Organization {
        let now = Utc::now();
        Organization {
            id,
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
            contact_email: Some("contact@treyve.test".to_owned()),
            contact_phone: None,
            insurance_mention: None,
            quote_number_prefix: "DEV".to_owned(),
            invoice_number_prefix: "FAC".to_owned(),
            field_clock_enabled: false,
            vat_on_debits: false,
            deleted_at: None,
            created_at: now,
            updated_at: now,
        }
    }

    struct Mocks {
        models: MockDoeModelRepository,
        projects: MockProjectRepository,
        customers: MockCustomerRepository,
        contexts: MockCustomerContextRepository,
        organizations: MockOrganizationRepository,
    }

    fn mocks(fixture: &Fixture) -> Mocks {
        let model = fixture.model.clone();
        let mut models = MockDoeModelRepository::new();
        models.expect_find_by_id().returning(move |_| {
            let model = model.clone();
            Box::pin(async move { Ok(Some(model)) })
        });
        let project = fixture.project.clone();
        let mut projects = MockProjectRepository::new();
        projects.expect_find_by_id().returning(move |_| {
            let project = project.clone();
            Box::pin(async move { Ok(Some(project)) })
        });
        let customer = fixture.customer.clone();
        let mut customers = MockCustomerRepository::new();
        customers.expect_find_by_id().returning(move |_| {
            let customer = customer.clone();
            Box::pin(async move { Ok(Some(customer)) })
        });
        let customer_id = fixture.customer.id;
        let mut contexts = MockCustomerContextRepository::new();
        contexts.expect_find_by_id().returning(move |id| {
            let now = Utc::now();
            let context = CustomerContext {
                id,
                customer_id,
                label: "Chantier".to_owned(),
                address_line: Some("Avenue du 8 mai".to_owned()),
                postal_code: None,
                city: None,
                photo_key: None,
                deleted_at: None,
                created_at: now,
                updated_at: now,
            };
            Box::pin(async move { Ok(Some(context)) })
        });
        let organization = organization(fixture.organization_id);
        let mut organizations = MockOrganizationRepository::new();
        organizations.expect_find_by_id().returning(move |_| {
            let organization = organization.clone();
            Box::pin(async move { Ok(Some(organization)) })
        });
        Mocks {
            models,
            projects,
            customers,
            contexts,
            organizations,
        }
    }

    fn create_command(fixture: &Fixture) -> CreateDoeCommand {
        CreateDoeCommand {
            actor: actor(),
            organization_id: fixture.organization_id,
            project_id: fixture.project.id,
            model_id: fixture.model.id,
        }
    }

    #[tokio::test]
    async fn create_builds_sections_and_parties_from_the_project() {
        let fixture = fixture();
        let mut mocks = mocks(&fixture);
        let mut repository = MockDoeRepository::new();
        repository
            .expect_find_by_project()
            .returning(|_| Box::pin(async { Ok(None) }));
        repository.expect_insert().returning(|doe| {
            let doe = doe.clone();
            Box::pin(async move { Ok(doe) })
        });
        let mut service = DoeService::new(repository);

        let doe = service
            .create_from_model(
                create_command(&fixture),
                &mut mocks.models,
                &mut mocks.projects,
                &mut mocks.customers,
                &mut mocks.contexts,
                &mut mocks.organizations,
            )
            .await
            .unwrap();

        assert_eq!(doe.title, "Avenue du 8 mai");
        assert_eq!(doe.revision.as_str(), "A");
        let titles: Vec<(&str, bool)> = doe
            .sections
            .iter()
            .map(|section| (section.title.as_str(), section.enabled))
            .collect();
        assert_eq!(
            titles,
            vec![("Végétaux", true), ("Jeux", false), ("Photos", true)]
        );
        assert!(matches!(
            &doe.sections[1].content,
            DoeSectionContent::Supplies { supplies, .. } if supplies.is_empty()
        ));
        assert_ne!(doe.sections[0].id, doe.sections[1].id);
        assert_eq!(doe.parties.len(), 2);
        assert_eq!(doe.parties[0].role, PartyRole::ProjectOwner);
        assert_eq!(doe.parties[0].name, "Commune de Lapalisse");
        assert_eq!(doe.parties[0].address.as_deref(), Some("Avenue du 8 mai"));
        assert_eq!(doe.parties[0].phone.as_deref(), Some("0400000000"));
        assert_eq!(doe.parties[1].role, PartyRole::Contractor);
        assert_eq!(doe.parties[1].name, "Treyve Paysages");
        assert_eq!(
            doe.parties[1].address.as_deref(),
            Some("3 chemin des Vignes\n03120 Lapalisse")
        );
        assert_eq!(doe.parties[1].email.as_deref(), Some("contact@treyve.test"));
    }

    #[tokio::test]
    async fn create_refuses_when_the_project_has_a_doe() {
        let fixture = fixture();
        let mut mocks = mocks(&fixture);
        let existing = existing_doe(fixture.organization_id);
        let mut repository = MockDoeRepository::new();
        repository.expect_find_by_project().returning(move |_| {
            let existing = existing.clone();
            Box::pin(async move { Ok(Some(existing)) })
        });
        let mut service = DoeService::new(repository);

        let err = service
            .create_from_model(
                create_command(&fixture),
                &mut mocks.models,
                &mut mocks.projects,
                &mut mocks.customers,
                &mut mocks.contexts,
                &mut mocks.organizations,
            )
            .await
            .unwrap_err();

        assert!(matches!(err, CoreError::Conflict(_)));
    }

    #[tokio::test]
    async fn create_refuses_an_archived_model() {
        let mut fixture = fixture();
        fixture.model.archived_at = Some(Utc::now());
        let mut mocks = mocks(&fixture);
        let mut service = DoeService::new(MockDoeRepository::new());

        let err = service
            .create_from_model(
                create_command(&fixture),
                &mut mocks.models,
                &mut mocks.projects,
                &mut mocks.customers,
                &mut mocks.contexts,
                &mut mocks.organizations,
            )
            .await
            .unwrap_err();

        assert!(matches!(err, CoreError::Conflict(_)));
    }

    #[tokio::test]
    async fn create_refuses_a_project_of_another_organization() {
        let mut fixture = fixture();
        fixture.project.organization_id = OrganizationId(Uuid::new_v4());
        let mut mocks = mocks(&fixture);
        let mut service = DoeService::new(MockDoeRepository::new());

        let err = service
            .create_from_model(
                create_command(&fixture),
                &mut mocks.models,
                &mut mocks.projects,
                &mut mocks.customers,
                &mut mocks.contexts,
                &mut mocks.organizations,
            )
            .await
            .unwrap_err();

        assert!(matches!(err, CoreError::NotFound));
    }

    #[tokio::test]
    async fn create_refuses_a_model_of_another_organization() {
        let mut fixture = fixture();
        fixture.model.organization_id = OrganizationId(Uuid::new_v4());
        let mut mocks = mocks(&fixture);
        let mut service = DoeService::new(MockDoeRepository::new());

        let err = service
            .create_from_model(
                create_command(&fixture),
                &mut mocks.models,
                &mut mocks.projects,
                &mut mocks.customers,
                &mut mocks.contexts,
                &mut mocks.organizations,
            )
            .await
            .unwrap_err();

        assert!(matches!(err, CoreError::NotFound));
    }
}
