use common::{CoreError, generate_uuid_v7};

use crate::{
    AddressKind, ExpenseCategory, ExpenseCategoryId, MockExpenseCategoryRepository,
    MockSupplierRepository, OrganizationId, SupplierAddressInput, SupplierContactInput,
    SupplierField, SupplierProfileInput,
    domain::supplier_invoice::{
        commands::CreateSupplierCommand, supplier_service::SupplierService,
    },
};

fn create_command(name: &str, email: Option<&str>) -> CreateSupplierCommand {
    CreateSupplierCommand {
        actor: authz::Subject::system(),
        organization_id: OrganizationId(generate_uuid_v7()),
        name: name.to_owned(),
        registration_number: None,
        vat_number: None,
        address: Some(" 12 rue des Lilas ".to_owned()),
        phone: Some("  ".to_owned()),
        email: email.map(str::to_owned),
        profile: SupplierProfileInput::default(),
    }
}

fn inserting() -> MockSupplierRepository {
    let mut repo = MockSupplierRepository::new();
    repo.expect_insert().returning(|supplier| {
        let supplier = supplier.clone();
        Box::pin(async move { Ok(supplier) })
    });
    repo
}

fn service(
    repo: MockSupplierRepository,
) -> SupplierService<MockSupplierRepository, MockExpenseCategoryRepository> {
    SupplierService::new(repo, MockExpenseCategoryRepository::new())
}

#[tokio::test]
async fn create_normalizes_contact_details() {
    let mut service = service(inserting());

    let supplier = service
        .create_supplier(create_command(" Point P ", Some("contact@pointp.fr")))
        .await
        .unwrap()
        .unwrap();

    assert_eq!(supplier.name, "Point P");
    assert_eq!(supplier.address.as_deref(), Some("12 rue des Lilas"));
    assert_eq!(supplier.phone, None);
    assert_eq!(supplier.email.as_deref(), Some("contact@pointp.fr"));
}

#[tokio::test]
async fn create_refuses_a_blank_name_naming_the_field() {
    let mut service = service(MockSupplierRepository::new());

    let refusal = service
        .create_supplier(create_command("  ", None))
        .await
        .unwrap()
        .unwrap_err();

    assert_eq!(refusal.field, SupplierField::Name);
}

#[tokio::test]
async fn create_refuses_an_email_without_at() {
    let mut service = service(MockSupplierRepository::new());

    let refusal = service
        .create_supplier(create_command("Point P", Some("pointp.fr")))
        .await
        .unwrap()
        .unwrap_err();

    assert_eq!(refusal.field, SupplierField::Email);
}

#[tokio::test]
async fn a_supplier_registration_number_of_9_or_14_digits_or_empty_is_accepted() {
    for accepted in [
        None,
        Some(""),
        Some("  "),
        Some("732 829 320"),
        Some("73282932000074"),
    ] {
        let mut service = service(inserting());
        let mut command = create_command("Point P", None);
        command.registration_number = accepted.map(str::to_owned);

        let supplier = service.create_supplier(command).await.unwrap().unwrap();

        let digits: Option<String> = accepted
            .map(|value| value.chars().filter(|c| !c.is_whitespace()).collect())
            .filter(|value: &String| !value.is_empty());
        assert_eq!(supplier.registration_number, digits);
    }
}

#[tokio::test]
async fn a_supplier_registration_number_of_any_other_length_is_refused() {
    for refused in ["1234", "1234567890", "73282932000074 1", "7328293A0"] {
        let mut service = service(MockSupplierRepository::new());
        let mut command = create_command("Point P", None);
        command.registration_number = Some(refused.to_owned());

        let refusal = service.create_supplier(command).await.unwrap().unwrap_err();

        assert_eq!(
            refusal.field,
            SupplierField::RegistrationNumber,
            "{refused}"
        );
    }
}

#[tokio::test]
async fn payment_terms_go_from_0_to_999_days() {
    for (days, accepted) in [(0, true), (999, true), (1000, false), (-1, false)] {
        let mut service = service(if accepted {
            inserting()
        } else {
            MockSupplierRepository::new()
        });
        let mut command = create_command("Point P", None);
        command.profile.payment_terms_days = Some(days);

        let outcome = service.create_supplier(command).await.unwrap();

        assert_eq!(outcome.is_ok(), accepted, "{days}");
        if let Err(refusal) = outcome {
            assert_eq!(refusal.field, SupplierField::PaymentTermsDays);
        }
    }
}

#[tokio::test]
async fn a_supplier_stores_iban_notes_several_contacts_and_typed_addresses() {
    let org = OrganizationId(generate_uuid_v7());
    let category = ExpenseCategory {
        id: ExpenseCategoryId(generate_uuid_v7()),
        organization_id: org,
        name: "Matériaux".to_owned(),
        hue: 60,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };
    let category_id = category.id;
    let mut categories = MockExpenseCategoryRepository::new();
    categories.expect_find_by_id().returning(move |_| {
        let category = category.clone();
        Box::pin(async move { Ok(Some(category)) })
    });
    let mut service = SupplierService::new(inserting(), categories);
    let mut command = create_command("Point P", None);
    command.organization_id = org;
    command.profile = SupplierProfileInput {
        iban: Some(" FR76 3000 6000 0112 3456 7890 189 ".to_owned()),
        payment_terms_days: Some(45),
        default_category_id: Some(category_id),
        notes: Some(" Livraison le mardi ".to_owned()),
        contacts: vec![
            SupplierContactInput {
                first_name: "Jean".to_owned(),
                last_name: "Bernard".to_owned(),
                email: "jean@pointp.fr".to_owned(),
                ..Default::default()
            },
            SupplierContactInput {
                last_name: "Compta".to_owned(),
                mobile: "0601020304".to_owned(),
                ..Default::default()
            },
        ],
        addresses: vec![
            SupplierAddressInput {
                kind: AddressKind::Billing,
                label: "Siège".to_owned(),
                street: "1 rue A".to_owned(),
                line2: String::new(),
                postal_code: "44000".to_owned(),
                city: "Nantes".to_owned(),
                country: "France".to_owned(),
            },
            SupplierAddressInput {
                kind: AddressKind::Site,
                label: "Dépôt".to_owned(),
                street: "2 rue B".to_owned(),
                line2: "Bât. 3".to_owned(),
                postal_code: "44800".to_owned(),
                city: "Saint-Herblain".to_owned(),
                country: "France".to_owned(),
            },
        ],
    };

    let supplier = service.create_supplier(command).await.unwrap().unwrap();

    assert_eq!(
        supplier.profile.iban.as_deref(),
        Some("FR76 3000 6000 0112 3456 7890 189")
    );
    assert_eq!(supplier.profile.payment_terms_days, Some(45));
    assert_eq!(supplier.profile.default_category_id, Some(category_id));
    assert_eq!(
        supplier.profile.notes.as_deref(),
        Some("Livraison le mardi")
    );
    assert_eq!(supplier.profile.contacts.len(), 2);
    assert_eq!(supplier.profile.contacts[1].mobile, "0601020304");
    assert_eq!(supplier.profile.addresses.len(), 2);
    assert_eq!(supplier.profile.addresses[1].kind, AddressKind::Site);
    assert_eq!(supplier.profile.addresses[1].line2, "Bât. 3");
    assert_eq!(supplier.profile.archived_at, None);
}

#[tokio::test]
async fn a_contact_with_an_invalid_email_is_refused() {
    let mut service = service(MockSupplierRepository::new());
    let mut command = create_command("Point P", None);
    command.profile.contacts = vec![SupplierContactInput {
        email: "pas un email".to_owned(),
        ..Default::default()
    }];

    let refusal = service.create_supplier(command).await.unwrap().unwrap_err();

    assert_eq!(refusal.field, SupplierField::Contacts);
}

#[tokio::test]
async fn a_default_category_of_another_organization_is_not_found() {
    let foreign = ExpenseCategory {
        id: ExpenseCategoryId(generate_uuid_v7()),
        organization_id: OrganizationId(generate_uuid_v7()),
        name: "Autres".to_owned(),
        hue: 0,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };
    let foreign_id = foreign.id;
    let mut categories = MockExpenseCategoryRepository::new();
    categories.expect_find_by_id().returning(move |_| {
        let category = foreign.clone();
        Box::pin(async move { Ok(Some(category)) })
    });
    let mut service = SupplierService::new(MockSupplierRepository::new(), categories);
    let mut command = create_command("Point P", None);
    command.profile.default_category_id = Some(foreign_id);

    let result = service.create_supplier(command).await;

    assert!(matches!(result, Err(CoreError::NotFound)));
}

#[tokio::test]
async fn archiving_sets_archived_at_and_restoring_clears_it() {
    let mut repo = MockSupplierRepository::new();
    repo.expect_find_by_id().returning(|id| {
        Box::pin(async move {
            Ok(Some(crate::Supplier {
                id,
                organization_id: OrganizationId(generate_uuid_v7()),
                name: "Point P".to_owned(),
                registration_number: None,
                vat_number: None,
                address: None,
                phone: None,
                email: None,
                profile: crate::SupplierProfile::default(),
                deleted_at: None,
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
            }))
        })
    });
    repo.expect_set_archived()
        .returning(|id, archived_at, updated_at| {
            Box::pin(async move {
                Ok(crate::Supplier {
                    id,
                    organization_id: OrganizationId(generate_uuid_v7()),
                    name: "Point P".to_owned(),
                    registration_number: None,
                    vat_number: None,
                    address: None,
                    phone: None,
                    email: None,
                    profile: crate::SupplierProfile {
                        archived_at,
                        ..Default::default()
                    },
                    deleted_at: None,
                    created_at: updated_at,
                    updated_at,
                })
            })
        });
    let mut service = service(repo);
    let id = crate::SupplierId(generate_uuid_v7());
    let now = chrono::Utc::now();

    let archived = service.set_archived(id, true, now).await.unwrap();
    let restored = service.set_archived(id, false, now).await.unwrap();

    assert_eq!(archived.profile.archived_at, Some(now));
    assert_eq!(restored.profile.archived_at, None);
}
