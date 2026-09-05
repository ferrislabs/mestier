#[cfg(test)]
#[allow(clippy::module_inception)]
mod tests {

    use chrono::{Duration, Utc};
    use common::{CoreError, OrganizationId, UserId, generate_uuid_v7};
    use sqlx::PgPool;

    use crate::application::test_support::{dev_pool, purge};
    use crate::application::{MestierUseCase, default_authorizer};
    use crate::domain::project::commands::{CreateProjectCommand, UpdateProjectCommand};
    use crate::domain::task::commands::{CreateTaskCommand, PatchTaskCommand};
    use crate::infrastructure::realtime::EventHub;

    async fn make_pool() -> PgPool {
        dev_pool().await
    }

    fn make_usecase(pool: PgPool) -> MestierUseCase {
        MestierUseCase::new(pool, default_authorizer(), EventHub::new())
    }

    struct Fixture {
        organization_id: OrganizationId,
        owner_id: UserId,
    }

    /// An owner and an organization. A project needs nothing else, which is
    /// the point of the entity: no customer, no quote, no member.
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

    async fn cleanup(pool: &PgPool, organization_id: OrganizationId, user_ids: &[UserId]) {
        purge(
            pool,
            "DELETE FROM tasks WHERE org_id = $1",
            organization_id.0,
        )
        .await;
        purge(
            pool,
            "DELETE FROM chat.channels WHERE org_id = $1",
            organization_id.0,
        )
        .await;
        purge(
            pool,
            "DELETE FROM projects WHERE org_id = $1",
            organization_id.0,
        )
        .await;
        purge(
            pool,
            "DELETE FROM organizations WHERE id = $1",
            organization_id.0,
        )
        .await;
        for uid in user_ids {
            purge(pool, "DELETE FROM users WHERE id = $1", uid.0).await;
        }
    }

    fn create_command(fixture: &Fixture, name: &str) -> CreateProjectCommand {
        CreateProjectCommand {
            actor: authz::Subject::system(),
            organization_id: fixture.organization_id,
            name: name.to_owned(),
            customer_id: None,
            customer_context_id: None,
            quote_id: None,
        }
    }

    fn root_task(fixture: &Fixture, title: &str) -> CreateTaskCommand {
        let now = Utc::now();

        CreateTaskCommand {
            actor: authz::Subject::system(),
            organization_id: fixture.organization_id,
            parent_task_id: None,
            title: title.to_owned(),
            description: None,
            starts_at: Some(now),
            ends_at: Some(now + Duration::hours(2)),
            all_day: false,
            blocks_availability: true,
            customer_id: None,
            customer_context_id: None,
            quote_id: None,
            project_id: None,
            expenses_cents: 0,
            expenses_label: None,
        }
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn a_project_with_no_customer_persists_and_reads_back_internal() {
        let pool = make_pool().await;
        let fixture = seed_fixture(&pool).await;
        let usecase = make_usecase(pool.clone());

        let created = usecase
            .create_project(create_command(&fixture, "Réunion hebdo"))
            .await
            .expect("create_project must succeed");

        assert!(created.is_internal());
        assert!(!created.is_archived());

        let fetched = usecase.get_project(created.id).await.unwrap();
        assert_eq!(fetched.name, "Réunion hebdo");
        assert!(fetched.customer_id.is_none());

        cleanup(&pool, fixture.organization_id, &[fixture.owner_id]).await;
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn listing_hides_archived_projects_unless_asked() {
        let pool = make_pool().await;
        let fixture = seed_fixture(&pool).await;
        let usecase = make_usecase(pool.clone());

        let kept = usecase
            .create_project(create_command(&fixture, "Chantier en cours"))
            .await
            .unwrap();
        let retired = usecase
            .create_project(create_command(&fixture, "Chantier fini"))
            .await
            .unwrap();

        usecase
            .archive_project(authz::Subject::system(), retired.id)
            .await
            .unwrap();

        let (active, active_total) = usecase
            .list_projects(fixture.organization_id, None, None, false, 20, 0)
            .await
            .unwrap();
        assert_eq!(active_total, 1);
        assert_eq!(active[0].id, kept.id);

        let (all, all_total) = usecase
            .list_projects(fixture.organization_id, None, None, true, 20, 0)
            .await
            .unwrap();
        assert_eq!(all_total, 2);

        // An archived project still resolves by id: a report over the period it
        // was worked has to be able to name it.
        let fetched = usecase.get_project(retired.id).await.unwrap();
        assert!(fetched.is_archived());

        usecase
            .restore_project(authz::Subject::system(), retired.id)
            .await
            .unwrap();
        assert!(!usecase.get_project(retired.id).await.unwrap().is_archived());
        let _ = all;

        cleanup(&pool, fixture.organization_id, &[fixture.owner_id]).await;
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn a_subtask_can_join_a_project_its_parent_does_not_name() {
        let pool = make_pool().await;
        let fixture = seed_fixture(&pool).await;
        let usecase = make_usecase(pool.clone());

        let project = usecase
            .create_project(create_command(&fixture, "Projet interne"))
            .await
            .unwrap();
        let parent = usecase
            .create_task(root_task(&fixture, "Racine"))
            .await
            .unwrap();

        let mut child_command = root_task(&fixture, "Envoyer le rapport à Brigitte");
        child_command.parent_task_id = Some(parent.id);
        child_command.starts_at = None;
        child_command.ends_at = None;
        child_command.project_id = Some(project.id);
        let child = usecase.create_task(child_command).await.unwrap();

        assert_eq!(child.project_id, Some(project.id));
        assert!(parent.project_id.is_none());

        cleanup(&pool, fixture.organization_id, &[fixture.owner_id]).await;
    }

    /// The composite foreign key on `(project_id, org_id)` makes this
    /// impossible at the schema level, but it would surface as a raw database
    /// error, so the application seam resolves the project first. What the
    /// caller must see is a 404, not a 500.
    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn attaching_another_organizations_project_is_not_found() {
        let pool = make_pool().await;
        let mine = seed_fixture(&pool).await;
        let theirs = seed_fixture(&pool).await;
        let usecase = make_usecase(pool.clone());

        let foreign_project = usecase
            .create_project(create_command(&theirs, "Leur projet"))
            .await
            .unwrap();
        let task = usecase
            .create_task(root_task(&mine, "Ma tâche"))
            .await
            .unwrap();

        let mut patch = PatchTaskCommand::new(task.id, authz::Subject::system());
        patch.project_id = Some(Some(foreign_project.id));

        let err = usecase.patch_task(patch).await.unwrap_err();
        assert!(matches!(err, CoreError::NotFound), "got {err:?}");

        // And nothing was written: the guard runs before the update.
        assert!(
            usecase
                .get_task(task.id)
                .await
                .unwrap()
                .project_id
                .is_none()
        );

        let mut creation = root_task(&mine, "Autre tâche");
        creation.project_id = Some(foreign_project.id);
        assert!(matches!(
            usecase.create_task(creation).await.unwrap_err(),
            CoreError::NotFound
        ));

        cleanup(&pool, mine.organization_id, &[mine.owner_id]).await;
        cleanup(&pool, theirs.organization_id, &[theirs.owner_id]).await;
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn updating_a_project_clears_the_customer_when_sent_empty() {
        let pool = make_pool().await;
        let fixture = seed_fixture(&pool).await;
        let usecase = make_usecase(pool.clone());

        let created = usecase
            .create_project(create_command(&fixture, "Projet"))
            .await
            .unwrap();

        let updated = usecase
            .update_project(UpdateProjectCommand {
                actor: authz::Subject::system(),
                id: created.id,
                name: "Projet renommé".to_owned(),
                customer_id: None,
                customer_context_id: None,
                quote_id: None,
            })
            .await
            .unwrap();

        assert_eq!(updated.name, "Projet renommé");
        assert!(updated.is_internal());

        cleanup(&pool, fixture.organization_id, &[fixture.owner_id]).await;
    }

    // -----------------------------------------------------------------
    // #345 — a project's one channel
    // -----------------------------------------------------------------

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn creating_a_projects_channel_names_it_from_the_project_by_default() {
        let pool = make_pool().await;
        let fixture = seed_fixture(&pool).await;
        let usecase = make_usecase(pool.clone());

        let project = usecase
            .create_project(create_command(&fixture, "Toiture Dupont"))
            .await
            .unwrap();

        let channel = usecase
            .create_project_channel(project.id, None)
            .await
            .unwrap();

        assert_eq!(channel.name, "Toiture Dupont");
        assert_eq!(channel.project_id, Some(project.id));
        assert!(!channel.archived);

        let fetched = usecase.get_project_channel(project.id).await.unwrap();
        assert_eq!(fetched.id, channel.id);

        cleanup(&pool, fixture.organization_id, &[fixture.owner_id]).await;
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn a_project_cannot_grow_a_second_channel() {
        let pool = make_pool().await;
        let fixture = seed_fixture(&pool).await;
        let usecase = make_usecase(pool.clone());

        let project = usecase
            .create_project(create_command(&fixture, "Chantier Martin"))
            .await
            .unwrap();

        usecase
            .create_project_channel(project.id, None)
            .await
            .unwrap();

        let err = usecase
            .create_project_channel(project.id, Some("second-channel".to_owned()))
            .await
            .unwrap_err();

        assert!(matches!(err, CoreError::Conflict(_)), "got {err:?}");

        cleanup(&pool, fixture.organization_id, &[fixture.owner_id]).await;
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn archiving_a_project_archives_its_channel_and_restoring_reverses_it() {
        let pool = make_pool().await;
        let fixture = seed_fixture(&pool).await;
        let usecase = make_usecase(pool.clone());

        let project = usecase
            .create_project(create_command(&fixture, "Chantier Leroy"))
            .await
            .unwrap();
        usecase
            .create_project_channel(project.id, None)
            .await
            .unwrap();

        usecase
            .archive_project(authz::Subject::system(), project.id)
            .await
            .unwrap();
        assert!(
            usecase
                .get_project_channel(project.id)
                .await
                .unwrap()
                .archived,
            "archiving the project must archive its channel"
        );

        usecase
            .restore_project(authz::Subject::system(), project.id)
            .await
            .unwrap();
        assert!(
            !usecase
                .get_project_channel(project.id)
                .await
                .unwrap()
                .archived,
            "restoring the project must un-archive its channel"
        );

        cleanup(&pool, fixture.organization_id, &[fixture.owner_id]).await;
    }

    /// Most projects never grow a channel — archiving one must not fail or
    /// try to reach a channel that does not exist.
    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn archiving_a_project_with_no_channel_is_unaffected() {
        let pool = make_pool().await;
        let fixture = seed_fixture(&pool).await;
        let usecase = make_usecase(pool.clone());

        let project = usecase
            .create_project(create_command(&fixture, "Projet sans canal"))
            .await
            .unwrap();

        usecase
            .archive_project(authz::Subject::system(), project.id)
            .await
            .unwrap();
        assert!(usecase.get_project(project.id).await.unwrap().is_archived());
        assert!(matches!(
            usecase.get_project_channel(project.id).await.unwrap_err(),
            CoreError::NotFound
        ));

        cleanup(&pool, fixture.organization_id, &[fixture.owner_id]).await;
    }
}
