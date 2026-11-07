#[cfg(test)]
#[allow(clippy::module_inception)]
mod tests {
    use common::{CoreError, OrganizationId, generate_uuid_v7};
    use sqlx::PgPool;

    use crate::application::test_support::{dev_pool, purge};
    use crate::application::{MestierUseCase, default_authorizer};
    use crate::domain::supplier_invoice::commands::{CreateSupplierCommand, UpdateSupplierCommand};
    use crate::infrastructure::realtime::EventHub;
    use crate::{SupplierAddressInput, SupplierContactInput, SupplierProfileInput};

    fn make_usecase(pool: PgPool) -> MestierUseCase {
        MestierUseCase::new(pool, default_authorizer(), EventHub::new())
    }

    async fn seed_organization(pool: &PgPool) -> (OrganizationId, uuid::Uuid) {
        let owner_id = generate_uuid_v7();
        sqlx::query(
            r#"INSERT INTO users (id, email, username, display_name, sub) VALUES ($1, $2, $3, $4, $5)"#,
        )
        .bind(owner_id)
        .bind(format!("owner-{owner_id}@example.com"))
        .bind(format!("owner-{owner_id}"))
        .bind("Owner User")
        .bind(format!("sub-owner-{owner_id}"))
        .execute(pool)
        .await
        .unwrap();

        let org_id = generate_uuid_v7();
        sqlx::query(
            r#"INSERT INTO organizations (id, name, slug, owner_id) VALUES ($1, $2, $3, $4)"#,
        )
        .bind(org_id)
        .bind("Test Org")
        .bind(format!("test-org-{org_id}"))
        .bind(owner_id)
        .execute(pool)
        .await
        .unwrap();

        (OrganizationId(org_id), owner_id)
    }

    async fn cleanup(pool: &PgPool, organization_id: OrganizationId, owner_id: uuid::Uuid) {
        purge(
            pool,
            "DELETE FROM suppliers WHERE org_id = $1",
            organization_id.0,
        )
        .await;
        purge(
            pool,
            "DELETE FROM organizations WHERE id = $1",
            organization_id.0,
        )
        .await;
        purge(pool, "DELETE FROM users WHERE id = $1", owner_id).await;
    }

    fn create_command(organization_id: OrganizationId, name: &str) -> CreateSupplierCommand {
        CreateSupplierCommand {
            actor: authz::Subject::system(),
            organization_id,
            name: name.to_owned(),
            registration_number: None,
            vat_number: None,
            address: Some("12 rue des Lilas, 75011 Paris".to_owned()),
            phone: Some("01 23 45 67 89".to_owned()),
            email: Some("contact@example.com".to_owned()),
            profile: SupplierProfileInput::default(),
        }
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn supplier_create_persists_contact_details() {
        let pool = dev_pool().await;
        let (organization_id, owner_id) = seed_organization(&pool).await;
        let usecase = make_usecase(pool.clone());

        let created = usecase
            .create_supplier(create_command(organization_id, "Point P"))
            .await
            .unwrap()
            .unwrap();
        let read = usecase.get_supplier(created.id).await.unwrap();

        assert_eq!(
            read.address.as_deref(),
            Some("12 rue des Lilas, 75011 Paris")
        );
        assert_eq!(read.phone.as_deref(), Some("01 23 45 67 89"));
        assert_eq!(read.email.as_deref(), Some("contact@example.com"));

        cleanup(&pool, organization_id, owner_id).await;
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn supplier_list_is_ordered_by_name_and_excludes_deleted() {
        let pool = dev_pool().await;
        let (organization_id, owner_id) = seed_organization(&pool).await;
        let usecase = make_usecase(pool.clone());

        usecase
            .create_supplier(create_command(organization_id, "Zolpan"))
            .await
            .unwrap()
            .unwrap();
        usecase
            .create_supplier(create_command(organization_id, "Brico"))
            .await
            .unwrap()
            .unwrap();
        let removed = usecase
            .create_supplier(create_command(organization_id, "Aaa"))
            .await
            .unwrap()
            .unwrap();
        sqlx::query("UPDATE suppliers SET deleted_at = now() WHERE id = $1")
            .bind(removed.id.0)
            .execute(&pool)
            .await
            .unwrap();

        let (items, total) = usecase
            .list_suppliers(organization_id, 10, 0)
            .await
            .unwrap();

        let names: Vec<&str> = items.iter().map(|s| s.name.as_str()).collect();
        assert_eq!(names, vec!["Brico", "Zolpan"]);
        assert_eq!(total, 2);

        cleanup(&pool, organization_id, owner_id).await;
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn supplier_update_rewrites_contact_details() {
        let pool = dev_pool().await;
        let (organization_id, owner_id) = seed_organization(&pool).await;
        let usecase = make_usecase(pool.clone());
        let created = usecase
            .create_supplier(create_command(organization_id, "Point P"))
            .await
            .unwrap()
            .unwrap();

        let updated = usecase
            .update_supplier(UpdateSupplierCommand {
                actor: authz::Subject::system(),
                id: created.id,
                name: "Point P Pro".to_owned(),
                registration_number: Some("732 829 320".to_owned()),
                vat_number: None,
                address: None,
                phone: Some("06 00 00 00 00".to_owned()),
                email: None,
                profile: SupplierProfileInput::default(),
            })
            .await
            .unwrap()
            .unwrap();

        assert_eq!(updated.name, "Point P Pro");
        assert_eq!(updated.registration_number.as_deref(), Some("732829320"));
        assert_eq!(updated.address, None);
        assert_eq!(updated.email, None);
        assert_eq!(updated.phone.as_deref(), Some("06 00 00 00 00"));

        let missing = usecase
            .update_supplier(UpdateSupplierCommand {
                actor: authz::Subject::system(),
                id: crate::SupplierId(generate_uuid_v7()),
                name: "X".to_owned(),
                registration_number: None,
                vat_number: None,
                address: None,
                phone: None,
                email: None,
                profile: SupplierProfileInput::default(),
            })
            .await;
        assert!(matches!(missing, Err(CoreError::NotFound)));

        cleanup(&pool, organization_id, owner_id).await;
    }

    fn rich_profile(category_id: crate::ExpenseCategoryId) -> SupplierProfileInput {
        SupplierProfileInput {
            iban: Some("FR76 3000 6000 0112 3456 7890 189".to_owned()),
            payment_terms_days: Some(45),
            default_category_id: Some(category_id),
            notes: Some("Livraison le mardi".to_owned()),
            contacts: vec![
                SupplierContactInput {
                    first_name: "Jean".to_owned(),
                    last_name: "Bernard".to_owned(),
                    role: "Commercial".to_owned(),
                    email: "jean@pointp.fr".to_owned(),
                    phone: "0240000000".to_owned(),
                    mobile: "0600000000".to_owned(),
                },
                SupplierContactInput {
                    last_name: "Compta".to_owned(),
                    ..Default::default()
                },
            ],
            addresses: vec![
                SupplierAddressInput {
                    kind: crate::AddressKind::Billing,
                    label: "Siège".to_owned(),
                    street: "1 rue A".to_owned(),
                    line2: String::new(),
                    postal_code: "44000".to_owned(),
                    city: "Nantes".to_owned(),
                    country: "France".to_owned(),
                },
                SupplierAddressInput {
                    kind: crate::AddressKind::Site,
                    label: "Dépôt".to_owned(),
                    street: "2 rue B".to_owned(),
                    line2: "Bât. 3".to_owned(),
                    postal_code: "44800".to_owned(),
                    city: "Saint-Herblain".to_owned(),
                    country: "France".to_owned(),
                },
            ],
        }
    }

    async fn seed_category(
        pool: &PgPool,
        organization_id: OrganizationId,
    ) -> crate::ExpenseCategoryId {
        let id = generate_uuid_v7();
        sqlx::query("INSERT INTO expense_categories (id, org_id, name, hue) VALUES ($1, $2, 'Matériaux', 60)")
            .bind(id)
            .bind(organization_id.0)
            .execute(pool)
            .await
            .unwrap();
        crate::ExpenseCategoryId(id)
    }

    async fn drop_categories(pool: &PgPool, organization_id: OrganizationId) {
        sqlx::query("UPDATE suppliers SET default_category_id = NULL WHERE org_id = $1")
            .bind(organization_id.0)
            .execute(pool)
            .await
            .unwrap();
        sqlx::query("DELETE FROM supplier_contacts WHERE org_id = $1")
            .bind(organization_id.0)
            .execute(pool)
            .await
            .unwrap();
        sqlx::query("DELETE FROM supplier_addresses WHERE org_id = $1")
            .bind(organization_id.0)
            .execute(pool)
            .await
            .unwrap();
        sqlx::query("DELETE FROM expense_categories WHERE org_id = $1")
            .bind(organization_id.0)
            .execute(pool)
            .await
            .unwrap();
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn supplier_profile_round_trips_replaces_contacts_and_archives() {
        let pool = dev_pool().await;
        let (organization_id, owner_id) = seed_organization(&pool).await;
        let category_id = seed_category(&pool, organization_id).await;
        let usecase = make_usecase(pool.clone());
        let mut command = create_command(organization_id, "Point P");
        command.profile = rich_profile(category_id);

        let created = usecase.create_supplier(command).await.unwrap().unwrap();
        let read = usecase.get_supplier(created.id).await.unwrap();

        assert_eq!(
            read.profile.iban.as_deref(),
            Some("FR76 3000 6000 0112 3456 7890 189")
        );
        assert_eq!(read.profile.payment_terms_days, Some(45));
        assert_eq!(read.profile.default_category_id, Some(category_id));
        assert_eq!(read.profile.notes.as_deref(), Some("Livraison le mardi"));
        assert_eq!(read.profile.contacts.len(), 2);
        assert_eq!(read.profile.contacts[0].mobile, "0600000000");
        assert_eq!(read.profile.contacts[1].last_name, "Compta");
        assert_eq!(read.profile.addresses.len(), 2);
        assert_eq!(read.profile.addresses[1].kind, crate::AddressKind::Site);
        assert_eq!(read.profile.addresses[1].line2, "Bât. 3");
        assert_eq!(read.profile.archived_at, None);

        let mut smaller = rich_profile(category_id);
        smaller.contacts.truncate(1);
        smaller.addresses.clear();
        smaller.payment_terms_days = None;
        let updated = usecase
            .update_supplier(UpdateSupplierCommand {
                actor: authz::Subject::system(),
                id: created.id,
                name: "Point P".to_owned(),
                registration_number: None,
                vat_number: None,
                address: None,
                phone: None,
                email: None,
                profile: smaller,
            })
            .await
            .unwrap()
            .unwrap();
        assert_eq!(updated.profile.contacts.len(), 1);
        assert!(updated.profile.addresses.is_empty());
        assert_eq!(updated.profile.payment_terms_days, None);

        let archived = usecase
            .archive_supplier(created.id, true, authz::Subject::system())
            .await
            .unwrap();
        assert!(archived.profile.archived_at.is_some());
        let (listed, total) = usecase
            .list_suppliers(organization_id, 10, 0)
            .await
            .unwrap();
        assert_eq!(total, 1);
        assert_eq!(listed[0].profile.contacts.len(), 1);
        assert!(listed[0].profile.archived_at.is_some());

        let edited_while_archived = usecase
            .update_supplier(UpdateSupplierCommand {
                actor: authz::Subject::system(),
                id: created.id,
                name: "Point P bis".to_owned(),
                registration_number: None,
                vat_number: None,
                address: None,
                phone: None,
                email: None,
                profile: SupplierProfileInput::default(),
            })
            .await
            .unwrap()
            .unwrap();
        assert!(edited_while_archived.profile.archived_at.is_some());

        let restored = usecase
            .archive_supplier(created.id, false, authz::Subject::system())
            .await
            .unwrap();
        assert_eq!(restored.profile.archived_at, None);

        drop_categories(&pool, organization_id).await;
        cleanup(&pool, organization_id, owner_id).await;
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn a_default_category_of_another_organization_is_not_found_and_a_bad_siren_is_refused() {
        let pool = dev_pool().await;
        let (organization_id, owner_id) = seed_organization(&pool).await;
        let (other, other_owner) = seed_organization(&pool).await;
        let foreign = seed_category(&pool, other).await;
        let usecase = make_usecase(pool.clone());
        let mut command = create_command(organization_id, "Point P");
        command.profile = rich_profile(foreign);

        let result = usecase.create_supplier(command).await;
        assert!(matches!(result, Err(CoreError::NotFound)));

        let mut bad = create_command(organization_id, "Point P");
        bad.registration_number = Some("12345".to_owned());
        let refusal = usecase.create_supplier(bad).await.unwrap().unwrap_err();
        assert_eq!(refusal.field, crate::SupplierField::RegistrationNumber);

        drop_categories(&pool, other).await;
        cleanup(&pool, organization_id, owner_id).await;
        cleanup(&pool, other, other_owner).await;
    }
}
