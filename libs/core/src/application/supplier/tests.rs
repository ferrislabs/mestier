#[cfg(test)]
#[allow(clippy::module_inception)]
mod tests {
    use common::{CoreError, OrganizationId, generate_uuid_v7};
    use sqlx::PgPool;

    use crate::application::test_support::{dev_pool, purge};
    use crate::application::{MestierUseCase, default_authorizer};
    use crate::domain::supplier_invoice::commands::{CreateSupplierCommand, UpdateSupplierCommand};
    use crate::infrastructure::realtime::EventHub;

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
            .unwrap();
        usecase
            .create_supplier(create_command(organization_id, "Brico"))
            .await
            .unwrap();
        let removed = usecase
            .create_supplier(create_command(organization_id, "Aaa"))
            .await
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
            .unwrap();

        let updated = usecase
            .update_supplier(UpdateSupplierCommand {
                actor: authz::Subject::system(),
                id: created.id,
                name: "Point P Pro".to_owned(),
                registration_number: Some("RCS 123".to_owned()),
                vat_number: None,
                address: None,
                phone: Some("06 00 00 00 00".to_owned()),
                email: None,
            })
            .await
            .unwrap();

        assert_eq!(updated.name, "Point P Pro");
        assert_eq!(updated.registration_number.as_deref(), Some("RCS 123"));
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
            })
            .await;
        assert!(matches!(missing, Err(CoreError::NotFound)));

        cleanup(&pool, organization_id, owner_id).await;
    }
}
