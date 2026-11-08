#[cfg(test)]
#[allow(clippy::module_inception)]
mod tests {
    use common::{OrganizationId, generate_uuid_v7};
    use sqlx::PgPool;

    use crate::application::test_support::{dev_pool, purge};
    use crate::application::{MestierUseCase, default_authorizer};
    use crate::domain::surcharge::commands::{
        ArchiveSurchargeCommand, CreateSurchargeCommand, ListSurchargesCommand, SurchargeDraft,
        UpdateSurchargeCommand,
    };
    use crate::infrastructure::realtime::EventHub;
    use crate::{Surcharge, SurchargeError, SurchargeId, SurchargeKind};

    fn make_usecase(pool: PgPool) -> MestierUseCase {
        MestierUseCase::new(pool, default_authorizer(), EventHub::new())
    }

    struct Fixture {
        organization_id: OrganizationId,
        owner_id: uuid::Uuid,
    }

    async fn seed(pool: &PgPool) -> Fixture {
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
            "Surcharges Org",
            format!("surcharges-org-{org_id}"),
            owner_id,
        )
        .execute(pool)
        .await
        .unwrap();

        Fixture {
            organization_id: OrganizationId(org_id),
            owner_id,
        }
    }

    async fn cleanup(pool: &PgPool, fixture: &Fixture) {
        let org = fixture.organization_id.0;
        for statement in [
            "DELETE FROM surcharges WHERE org_id = $1",
            "DELETE FROM organizations WHERE id = $1",
        ] {
            purge(pool, statement, org).await;
        }
        purge(pool, "DELETE FROM users WHERE id = $1", fixture.owner_id).await;
    }

    async fn list(usecase: &MestierUseCase, fixture: &Fixture) -> Vec<Surcharge> {
        usecase
            .list_surcharges(ListSurchargesCommand {
                actor: authz::Subject::system(),
                organization_id: fixture.organization_id,
            })
            .await
            .unwrap()
    }

    fn draft(name: &str, rate_bp: i32) -> SurchargeDraft {
        SurchargeDraft {
            name: name.to_owned(),
            rate_bp,
            kind: SurchargeKind::Other,
        }
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn spec_rh_6_1_default_surcharges_exist_once_after_concurrent_first_reads() {
        let pool = dev_pool().await;
        let usecase = make_usecase(pool.clone());
        let fixture = seed(&pool).await;

        let (first, second, third) = tokio::join!(
            list(&usecase, &fixture),
            list(&usecase, &fixture),
            list(&usecase, &fixture),
        );
        let again = list(&usecase, &fixture).await;

        let names: Vec<&str> = again.iter().map(|s| s.name.as_str()).collect();
        assert_eq!(
            names,
            vec![
                "Heures supplémentaires",
                "Heures supplémentaires au-delà de 8 h",
                "Travail du dimanche",
                "Jour férié travaillé",
                "Travail de nuit",
            ]
        );
        let rates: Vec<i32> = again.iter().map(|s| s.rate.get()).collect();
        assert_eq!(rates, vec![2500, 5000, 10000, 10000, 2500]);
        assert_eq!((first.len(), second.len(), third.len()), (5, 5, 5));

        cleanup(&pool, &fixture).await;
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn a_surcharge_is_created_revised_archived_and_restored_without_deleting() {
        let pool = dev_pool().await;
        let usecase = make_usecase(pool.clone());
        let fixture = seed(&pool).await;

        let created = usecase
            .create_surcharge(CreateSurchargeCommand {
                actor: authz::Subject::system(),
                organization_id: fixture.organization_id,
                draft: draft(" Astreinte ", 3000),
            })
            .await
            .unwrap()
            .unwrap();
        let blank = usecase
            .create_surcharge(CreateSurchargeCommand {
                actor: authz::Subject::system(),
                organization_id: fixture.organization_id,
                draft: draft(" ", 3000),
            })
            .await
            .unwrap();
        let negative = usecase
            .update_surcharge(UpdateSurchargeCommand {
                actor: authz::Subject::system(),
                organization_id: fixture.organization_id,
                id: created.id,
                draft: draft("Astreinte", -1),
            })
            .await
            .unwrap();
        let revised = usecase
            .update_surcharge(UpdateSurchargeCommand {
                actor: authz::Subject::system(),
                organization_id: fixture.organization_id,
                id: created.id,
                draft: draft("Astreinte lourde", 4500),
            })
            .await
            .unwrap()
            .unwrap();
        let archive = |archived| ArchiveSurchargeCommand {
            actor: authz::Subject::system(),
            organization_id: fixture.organization_id,
            id: created.id,
            archived,
        };
        let archived = usecase.archive_surcharge(archive(true)).await.unwrap();
        let restored = usecase.archive_surcharge(archive(false)).await.unwrap();
        let missing = usecase
            .archive_surcharge(ArchiveSurchargeCommand {
                id: SurchargeId(generate_uuid_v7()),
                ..archive(true)
            })
            .await
            .unwrap_err();

        assert_eq!(created.name.as_str(), "Astreinte");
        assert_eq!(blank, Err(SurchargeError::NameEmpty));
        assert_eq!(negative, Err(SurchargeError::RateInvalid));
        assert_eq!(revised.rate.get(), 4500);
        assert!(archived.is_archived());
        assert!(!restored.is_archived());
        assert!(matches!(missing, common::CoreError::NotFound));
        assert_eq!(list(&usecase, &fixture).await.len(), 6);

        cleanup(&pool, &fixture).await;
    }
}
