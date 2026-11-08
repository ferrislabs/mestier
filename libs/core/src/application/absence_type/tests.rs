#[cfg(test)]
#[allow(clippy::module_inception)]
mod tests {
    use common::{OrganizationId, generate_uuid_v7};
    use sqlx::PgPool;

    use crate::application::test_support::{dev_pool, purge};
    use crate::application::{MestierUseCase, default_authorizer};
    use crate::domain::absence_type::commands::{
        AbsenceTypeDraft, ArchiveAbsenceTypeCommand, CreateAbsenceTypeCommand,
        ListAbsenceTypesCommand, UpdateAbsenceTypeCommand,
    };
    use crate::infrastructure::realtime::EventHub;
    use crate::{AbsenceKind, AbsenceTypeError, CountMode};

    fn make_usecase(pool: PgPool) -> MestierUseCase {
        MestierUseCase::new(pool, default_authorizer(), EventHub::new())
    }

    async fn seed_organization(pool: &PgPool) -> (OrganizationId, uuid::Uuid) {
        let owner_id = generate_uuid_v7();
        sqlx::query!(
            r#"INSERT INTO users (id, email, username, display_name, sub)
               VALUES ($1, $2, $3, $4, $5)"#,
            owner_id,
            format!("owner-{owner_id}@example.com"),
            format!("owner-{owner_id}"),
            "Owner User",
            format!("sub-owner-{owner_id}"),
        )
        .execute(pool)
        .await
        .unwrap();
        let org_id = generate_uuid_v7();
        sqlx::query!(
            r#"INSERT INTO organizations (id, name, slug, owner_id)
               VALUES ($1, $2, $3, $4)"#,
            org_id,
            "Types Org",
            format!("types-org-{org_id}"),
            owner_id,
        )
        .execute(pool)
        .await
        .unwrap();

        (OrganizationId(org_id), owner_id)
    }

    async fn cleanup(pool: &PgPool, organization_id: OrganizationId, owner_id: uuid::Uuid) {
        purge(
            pool,
            "DELETE FROM absence_types WHERE org_id = $1",
            organization_id.0,
        )
        .await;
        purge(
            pool,
            "DELETE FROM leave_accounts WHERE org_id = $1",
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

    fn list(organization_id: OrganizationId) -> ListAbsenceTypesCommand {
        ListAbsenceTypesCommand {
            actor: authz::Subject::system(),
            organization_id,
        }
    }

    fn draft(name: &str) -> AbsenceTypeDraft {
        AbsenceTypeDraft {
            name: name.to_owned(),
            hue: 10,
            count_mode: CountMode::Calendar,
            account_id: None,
            paid: false,
            needs_approval: true,
            needs_proof: true,
            kind: AbsenceKind::Sick,
        }
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn spec_rh_3_1_the_first_read_creates_the_defaults_once() {
        let pool = dev_pool().await;
        let (organization_id, owner_id) = seed_organization(&pool).await;
        let usecase = make_usecase(pool.clone());

        let first = usecase
            .list_absence_types(list(organization_id))
            .await
            .unwrap();
        let second = usecase
            .list_absence_types(list(organization_id))
            .await
            .unwrap();

        let names: Vec<&str> = first.iter().map(|t| t.name.as_str()).collect();
        assert_eq!(
            names,
            [
                "Congés payés",
                "RTT",
                "Arrêt maladie",
                "Intempéries",
                "Événement familial",
                "Formation",
                "Congé sans solde",
            ]
        );
        assert_eq!(first, second);

        cleanup(&pool, organization_id, owner_id).await;
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn spec_rh_3_1_concurrent_first_reads_create_the_defaults_once() {
        let pool = dev_pool().await;
        let (organization_id, owner_id) = seed_organization(&pool).await;
        let usecase = make_usecase(pool.clone());

        let (a, b, c, d) = tokio::join!(
            usecase.list_absence_types(list(organization_id)),
            usecase.list_absence_types(list(organization_id)),
            usecase.list_absence_types(list(organization_id)),
            usecase.list_absence_types(list(organization_id)),
        );

        for read in [a, b, c, d] {
            assert_eq!(read.unwrap().len(), 7);
        }
        let stored = sqlx::query_scalar!(
            r#"SELECT COUNT(*) AS "count!" FROM absence_types WHERE org_id = $1"#,
            organization_id.0
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(stored, 7);

        cleanup(&pool, organization_id, owner_id).await;
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn types_are_created_updated_archived_and_restored_within_their_organization() {
        let pool = dev_pool().await;
        let (organization_id, owner_id) = seed_organization(&pool).await;
        let (other_id, other_owner) = seed_organization(&pool).await;
        let usecase = make_usecase(pool.clone());

        let created = usecase
            .create_absence_type(CreateAbsenceTypeCommand {
                actor: authz::Subject::system(),
                organization_id,
                draft: draft("Deuil"),
            })
            .await
            .unwrap()
            .unwrap();
        let updated = usecase
            .update_absence_type(UpdateAbsenceTypeCommand {
                actor: authz::Subject::system(),
                organization_id,
                id: created.id,
                draft: draft("Deuil proche"),
            })
            .await
            .unwrap()
            .unwrap();
        let blank = usecase
            .update_absence_type(UpdateAbsenceTypeCommand {
                actor: authz::Subject::system(),
                organization_id,
                id: created.id,
                draft: draft("  "),
            })
            .await
            .unwrap();
        let hue = usecase
            .create_absence_type(CreateAbsenceTypeCommand {
                actor: authz::Subject::system(),
                organization_id,
                draft: AbsenceTypeDraft {
                    hue: 400,
                    ..draft("Hors teinte")
                },
            })
            .await
            .unwrap();
        let foreign = usecase
            .archive_absence_type(ArchiveAbsenceTypeCommand {
                actor: authz::Subject::system(),
                organization_id: other_id,
                id: created.id,
                archived: true,
            })
            .await
            .unwrap_err();
        let archived = usecase
            .archive_absence_type(ArchiveAbsenceTypeCommand {
                actor: authz::Subject::system(),
                organization_id,
                id: created.id,
                archived: true,
            })
            .await
            .unwrap();
        let restored = usecase
            .archive_absence_type(ArchiveAbsenceTypeCommand {
                actor: authz::Subject::system(),
                organization_id,
                id: created.id,
                archived: false,
            })
            .await
            .unwrap();

        assert_eq!(updated.name.as_str(), "Deuil proche");
        assert_eq!(updated.kind, AbsenceKind::Sick);
        assert_eq!(blank, Err(AbsenceTypeError::NameEmpty));
        assert_eq!(hue, Err(AbsenceTypeError::HueOutOfRange));
        assert!(matches!(foreign, common::CoreError::NotFound));
        assert!(archived.archived_at.is_some());
        assert!(restored.archived_at.is_none());
        let seeded = usecase
            .list_absence_types(list(organization_id))
            .await
            .unwrap();
        assert_eq!(seeded.len(), 8);

        cleanup(&pool, organization_id, owner_id).await;
        cleanup(&pool, other_id, other_owner).await;
    }
}
