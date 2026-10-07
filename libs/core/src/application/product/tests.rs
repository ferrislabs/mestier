#[cfg(test)]
#[allow(clippy::module_inception)]
mod tests {
    use common::{CoreError, OrganizationId, generate_uuid_v7};
    use sqlx::PgPool;

    use crate::application::test_support::{dev_pool, purge};
    use crate::application::{MestierUseCase, default_authorizer};
    use crate::domain::product::commands::{CreateProductCommand, UpdateProductCommand};
    use crate::domain::supplier_invoice::commands::CreateSupplierCommand;
    use crate::infrastructure::realtime::EventHub;
    use crate::{Supplier, SupplierId};

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

    async fn cleanup(pool: &PgPool, organizations: &[(OrganizationId, uuid::Uuid)]) {
        for (organization_id, owner_id) in organizations {
            purge(
                pool,
                "DELETE FROM products WHERE org_id = $1",
                organization_id.0,
            )
            .await;
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
            purge(pool, "DELETE FROM users WHERE id = $1", *owner_id).await;
        }
    }

    async fn create_supplier(
        usecase: &MestierUseCase,
        organization_id: OrganizationId,
    ) -> Supplier {
        usecase
            .create_supplier(CreateSupplierCommand {
                actor: authz::Subject::system(),
                organization_id,
                name: "Point P".to_owned(),
                registration_number: None,
                vat_number: None,
                address: Some("12 rue des Lilas".to_owned()),
                phone: None,
                email: None,
            })
            .await
            .unwrap()
    }

    fn create_command(
        organization_id: OrganizationId,
        supplier_id: Option<SupplierId>,
    ) -> CreateProductCommand {
        CreateProductCommand {
            actor: authz::Subject::system(),
            organization_id,
            name: "Rosier".to_owned(),
            sku: None,
            unit: "u".to_owned(),
            unit_price_cents: 1200,
            default_vat_rate_bp: None,
            description: None,
            photo_keys: vec![],
            supplier_id,
            manufacturer: Some("Pépinières Martin".to_owned()),
            technical_specs: Some("Hauteur 60 cm, conteneur 3 L".to_owned()),
            datasheet_keys: vec!["sheets/rosier.pdf".to_owned()],
        }
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn product_technical_sheet_round_trips() {
        let pool = dev_pool().await;
        let org = seed_organization(&pool).await;
        let usecase = make_usecase(pool.clone());
        let supplier = create_supplier(&usecase, org.0).await;

        let created = usecase
            .create_product(create_command(org.0, Some(supplier.id)))
            .await
            .unwrap();
        let read = usecase.get_product(created.id).await.unwrap();

        assert_eq!(read.supplier_id, Some(supplier.id));
        assert_eq!(read.manufacturer.as_deref(), Some("Pépinières Martin"));
        assert_eq!(
            read.technical_specs.as_deref(),
            Some("Hauteur 60 cm, conteneur 3 L")
        );
        assert_eq!(read.datasheet_keys, vec!["sheets/rosier.pdf"]);

        let updated = usecase
            .update_product(UpdateProductCommand {
                actor: authz::Subject::system(),
                id: created.id,
                name: "Rosier".to_owned(),
                sku: None,
                unit: "u".to_owned(),
                unit_price_cents: 1200,
                default_vat_rate_bp: None,
                description: None,
                photo_keys: vec![],
                supplier_id: None,
                manufacturer: None,
                technical_specs: None,
                datasheet_keys: vec![],
            })
            .await
            .unwrap();
        assert_eq!(updated.supplier_id, None);
        assert_eq!(updated.manufacturer, None);
        assert!(updated.datasheet_keys.is_empty());

        cleanup(&pool, &[org]).await;
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn product_with_a_supplier_of_another_organization_is_not_found() {
        let pool = dev_pool().await;
        let org = seed_organization(&pool).await;
        let other = seed_organization(&pool).await;
        let usecase = make_usecase(pool.clone());
        let foreign = create_supplier(&usecase, other.0).await;

        let result = usecase
            .create_product(create_command(org.0, Some(foreign.id)))
            .await;

        assert!(matches!(result, Err(CoreError::NotFound)));

        cleanup(&pool, &[org, other]).await;
    }
}
