#[cfg(test)]
#[allow(clippy::module_inception)]
mod tests {
    use common::{OrganizationId, UserId, generate_uuid_v7};
    use sqlx::PgPool;

    use crate::application::test_support::{dev_pool, purge};
    use crate::application::{MestierUseCase, default_authorizer};
    use crate::domain::doe_model::commands::{
        CreateDoeModelFromStarterCommand, UpdateDoeModelCommand, UpdateDoeSettingsCommand,
    };
    use crate::infrastructure::realtime::EventHub;
    use crate::{
        DoeFont, DoeModelSection, DoeModelSectionKind, DoeSettings, DoeStyle, DoeTemplate,
        HexColor, StarterModel,
    };

    fn make_usecase(pool: PgPool) -> MestierUseCase {
        MestierUseCase::new(pool, default_authorizer(), EventHub::new())
    }

    struct Fixture {
        organization_id: OrganizationId,
        owner_id: UserId,
    }

    async fn seed_fixture(pool: &PgPool) -> Fixture {
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
            "Test Org",
            format!("test-org-{org_id}"),
            owner_id,
        )
        .execute(pool)
        .await
        .unwrap();

        Fixture {
            organization_id: OrganizationId(org_id),
            owner_id: UserId(owner_id),
        }
    }

    async fn cleanup(pool: &PgPool, fixture: &Fixture) {
        for statement in [
            "DELETE FROM doe_models WHERE org_id = $1",
            "DELETE FROM doe_settings WHERE org_id = $1",
            "DELETE FROM organizations WHERE id = $1",
        ] {
            purge(pool, statement, fixture.organization_id.0).await;
        }
        purge(pool, "DELETE FROM users WHERE id = $1", fixture.owner_id.0).await;
    }

    fn from_starter(fixture: &Fixture) -> CreateDoeModelFromStarterCommand {
        CreateDoeModelFromStarterCommand {
            actor: authz::Subject::system(),
            organization_id: fixture.organization_id,
            starter: StarterModel::Landscaping,
        }
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn editing_a_copy_never_touches_the_starter() {
        let pool = dev_pool().await;
        let fixture = seed_fixture(&pool).await;
        let usecase = make_usecase(pool.clone());

        let created = usecase
            .create_doe_model_from_starter(from_starter(&fixture))
            .await
            .unwrap();
        assert_eq!(created.name, "Paysagiste");
        assert_eq!(created.sections, StarterModel::Landscaping.sections());

        let edited = usecase
            .update_doe_model(UpdateDoeModelCommand {
                actor: authz::Subject::system(),
                id: created.id,
                name: "Paysage maison".to_owned(),
                description: None,
                sections: vec![DoeModelSection {
                    title: "Plans".to_owned(),
                    enabled: true,
                    kind: DoeModelSectionKind::Plans,
                }],
                style: Some(DoeStyle {
                    heading_font: DoeFont::Montserrat,
                    ..DoeStyle::default()
                }),
            })
            .await
            .unwrap();
        assert_eq!(edited.sections.len(), 1);
        assert_eq!(created.style, None);
        assert_eq!(
            edited.style.as_ref().map(|style| style.heading_font),
            Some(DoeFont::Montserrat)
        );

        let fetched = usecase.get_doe_model(created.id).await.unwrap();
        assert_eq!(fetched.name, "Paysage maison");
        assert_eq!(fetched.sections.len(), 1);
        assert_eq!(StarterModel::Landscaping.name(), "Paysagiste");
        assert_eq!(StarterModel::Landscaping.sections().len(), 7);

        cleanup(&pool, &fixture).await;
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn list_excludes_archived_unless_asked() {
        let pool = dev_pool().await;
        let fixture = seed_fixture(&pool).await;
        let usecase = make_usecase(pool.clone());

        let kept = usecase
            .create_doe_model_from_starter(from_starter(&fixture))
            .await
            .unwrap();
        let archived = usecase
            .create_doe_model_from_starter(from_starter(&fixture))
            .await
            .unwrap();
        usecase
            .archive_doe_model(authz::Subject::system(), archived.id)
            .await
            .unwrap();

        let (items, total) = usecase
            .list_doe_models(fixture.organization_id, false, 20, 0)
            .await
            .unwrap();
        assert_eq!(total, 1);
        assert_eq!(items[0].id, kept.id);

        let (_, total_all) = usecase
            .list_doe_models(fixture.organization_id, true, 20, 0)
            .await
            .unwrap();
        assert_eq!(total_all, 2);

        usecase
            .restore_doe_model(authz::Subject::system(), archived.id)
            .await
            .unwrap();
        let (_, restored) = usecase
            .list_doe_models(fixture.organization_id, false, 20, 0)
            .await
            .unwrap();
        assert_eq!(restored, 2);

        cleanup(&pool, &fixture).await;
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn settings_upsert_round_trips() {
        let pool = dev_pool().await;
        let fixture = seed_fixture(&pool).await;
        let usecase = make_usecase(pool.clone());

        let default = usecase
            .get_doe_settings(fixture.organization_id)
            .await
            .unwrap();
        assert_eq!(default, DoeSettings::default_for(fixture.organization_id));

        for (accent, logo) in [("#ABCDEF", Some("logos/a.png")), ("#123456", None)] {
            let saved = usecase
                .update_doe_settings(UpdateDoeSettingsCommand {
                    actor: authz::Subject::system(),
                    organization_id: fixture.organization_id,
                    style: DoeStyle {
                        primary: HexColor::try_from(accent.to_owned()).unwrap(),
                        logo_key: logo.map(str::to_owned),
                        footer_text: DoeTemplate::try_from("Treyve {date}".to_owned()).unwrap(),
                        ..DoeStyle::default()
                    },
                })
                .await
                .unwrap();
            let fetched = usecase
                .get_doe_settings(fixture.organization_id)
                .await
                .unwrap();
            assert_eq!(saved, fetched);
            assert_eq!(fetched.style.primary.as_str(), accent.to_ascii_lowercase());
            assert_eq!(fetched.style.logo_key.as_deref(), logo);
            assert_eq!(fetched.style.footer_text.as_str(), "Treyve {date}");
        }

        cleanup(&pool, &fixture).await;
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn a_model_stays_attributed_to_its_own_organization() {
        let pool = dev_pool().await;
        let owner = seed_fixture(&pool).await;
        let other = seed_fixture(&pool).await;
        let usecase = make_usecase(pool.clone());

        let created = usecase
            .create_doe_model_from_starter(from_starter(&owner))
            .await
            .unwrap();

        let loaded = usecase.get_doe_model(created.id).await.unwrap();
        assert_eq!(loaded.organization_id, owner.organization_id);
        assert_ne!(loaded.organization_id, other.organization_id);

        let (items, total) = usecase
            .list_doe_models(other.organization_id, true, 20, 0)
            .await
            .unwrap();
        assert_eq!(total, 0);
        assert!(items.is_empty());

        cleanup(&pool, &owner).await;
        cleanup(&pool, &other).await;
    }
}
