#[cfg(test)]
#[allow(clippy::module_inception)]
mod tests {

    use chrono::{Duration, Utc};
    use common::{OrganizationId, UserId, generate_uuid_v7};
    use sqlx::PgPool;

    use crate::application::{MestierUseCase, default_authorizer};
    use crate::domain::task_label::commands::{CreateTaskLabelCommand, UpdateTaskLabelCommand};
    use crate::infrastructure::realtime::EventHub;
    use crate::{CreateOrganizationCommand, CreateTaskCommand, PatchTaskCommand, TaskLabelId};

    async fn make_pool() -> PgPool {
        let url = std::env::var("DATABASE_URL")
            .expect("DATABASE_URL must be set to run task label integration tests");
        PgPool::connect(&url).await.unwrap()
    }

    fn make_usecase(pool: PgPool) -> MestierUseCase {
        MestierUseCase::new(pool, default_authorizer(), EventHub::new())
    }

    struct Fixture {
        organization_id: OrganizationId,
        owner_id: UserId,
    }

    /// Seeds an owner user and an organization — the minimal graph a task
    /// label needs to exist. Unlike `task`'s own fixture, no customer is
    /// seeded: labels and their links to tasks never depend on a chantier.
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

    /// Removes everything seeded under `organization_id`: labels and links
    /// cascade from `tasks`/`task_labels` themselves, so only those two plus
    /// the organization and its loose users need explicit cleanup.
    async fn cleanup(pool: &PgPool, organization_id: OrganizationId, user_ids: &[UserId]) {
        sqlx::query!(
            "DELETE FROM task_assignments WHERE org_id = $1",
            organization_id.0
        )
        .execute(pool)
        .await
        .ok();
        sqlx::query!("DELETE FROM tasks WHERE org_id = $1", organization_id.0)
            .execute(pool)
            .await
            .ok();
        sqlx::query!(
            "DELETE FROM task_labels WHERE org_id = $1",
            organization_id.0
        )
        .execute(pool)
        .await
        .ok();
        sqlx::query!("DELETE FROM organizations WHERE id = $1", organization_id.0)
            .execute(pool)
            .await
            .ok();
        for uid in user_ids {
            sqlx::query!("DELETE FROM users WHERE id = $1", uid.0)
                .execute(pool)
                .await
                .ok();
        }
    }

    fn create_label_command(organization_id: OrganizationId, name: &str) -> CreateTaskLabelCommand {
        CreateTaskLabelCommand {
            organization_id,
            name: name.to_owned(),
            color: "#DC2626".to_owned(),
        }
    }

    fn create_task_command(fixture: &Fixture, title: &str) -> CreateTaskCommand {
        let now = Utc::now();
        CreateTaskCommand {
            organization_id: fixture.organization_id,
            parent_task_id: None,
            title: title.to_owned(),
            description: None,
            starts_at: Some(now),
            ends_at: Some(now + Duration::hours(1)),
            all_day: false,
            blocks_availability: true,
            customer_id: None,
            customer_context_id: None,
            quote_id: None,
        }
    }

    async fn count_links_for_label(pool: &PgPool, label_id: TaskLabelId) -> i64 {
        sqlx::query_scalar!(
            r#"SELECT COUNT(*) AS "count!" FROM task_label_links WHERE label_id = $1"#,
            label_id.0,
        )
        .fetch_one(pool)
        .await
        .unwrap()
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn create_task_label_persists_and_is_retrievable() {
        let pool = make_pool().await;
        let fixture = seed_fixture(&pool).await;
        let usecase = make_usecase(pool.clone());

        let created = usecase
            .create_task_label(create_label_command(fixture.organization_id, "Urgent"))
            .await
            .expect("create_task_label must succeed");

        assert_eq!(created.name, "Urgent");
        assert_eq!(created.color, "#DC2626");

        let fetched = usecase
            .get_task_label(created.id)
            .await
            .expect("get_task_label must succeed");
        assert_eq!(fetched.id, created.id);
        assert_eq!(fetched.name, "Urgent");

        cleanup(&pool, fixture.organization_id, &[fixture.owner_id]).await;
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn duplicate_name_is_rejected_within_an_organization_but_accepted_in_another() {
        let pool = make_pool().await;
        let fixture_a = seed_fixture(&pool).await;
        let fixture_b = seed_fixture(&pool).await;
        let usecase = make_usecase(pool.clone());

        usecase
            .create_task_label(create_label_command(fixture_a.organization_id, "Urgent"))
            .await
            .expect("first creation in org A must succeed");

        let err = usecase
            .create_task_label(create_label_command(fixture_a.organization_id, "Urgent"))
            .await
            .expect_err("a duplicate name within the same organization must be rejected");
        assert!(matches!(err, common::CoreError::Conflict(_)));

        let created_in_b = usecase
            .create_task_label(create_label_command(fixture_b.organization_id, "Urgent"))
            .await
            .expect("the same name in a different organization must be accepted");
        assert_eq!(created_in_b.name, "Urgent");

        cleanup(&pool, fixture_a.organization_id, &[fixture_a.owner_id]).await;
        cleanup(&pool, fixture_b.organization_id, &[fixture_b.owner_id]).await;
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn list_task_labels_returns_only_the_organizations_own_labels() {
        let pool = make_pool().await;
        let fixture_a = seed_fixture(&pool).await;
        let fixture_b = seed_fixture(&pool).await;
        let usecase = make_usecase(pool.clone());

        usecase
            .create_task_label(create_label_command(fixture_a.organization_id, "Urgent"))
            .await
            .unwrap();
        usecase
            .create_task_label(create_label_command(fixture_b.organization_id, "Autre"))
            .await
            .unwrap();

        let labels = usecase
            .list_task_labels(fixture_a.organization_id)
            .await
            .expect("list_task_labels must succeed");

        assert_eq!(labels.len(), 1);
        assert_eq!(labels[0].name, "Urgent");
        assert_eq!(labels[0].organization_id, fixture_a.organization_id);

        cleanup(&pool, fixture_a.organization_id, &[fixture_a.owner_id]).await;
        cleanup(&pool, fixture_b.organization_id, &[fixture_b.owner_id]).await;
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn update_task_label_mutates_existing_label() {
        let pool = make_pool().await;
        let fixture = seed_fixture(&pool).await;
        let usecase = make_usecase(pool.clone());

        let created = usecase
            .create_task_label(create_label_command(fixture.organization_id, "Urgent"))
            .await
            .unwrap();

        let updated = usecase
            .update_task_label(UpdateTaskLabelCommand {
                id: created.id,
                name: "Prioritaire".to_owned(),
                color: "#059669".to_owned(),
            })
            .await
            .expect("update_task_label must succeed");

        assert_eq!(updated.name, "Prioritaire");
        assert_eq!(updated.color, "#059669");

        cleanup(&pool, fixture.organization_id, &[fixture.owner_id]).await;
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn delete_task_label_removes_it() {
        let pool = make_pool().await;
        let fixture = seed_fixture(&pool).await;
        let usecase = make_usecase(pool.clone());

        let created = usecase
            .create_task_label(create_label_command(fixture.organization_id, "Urgent"))
            .await
            .unwrap();

        usecase
            .delete_task_label(created.id)
            .await
            .expect("delete_task_label must succeed");

        let err = usecase
            .get_task_label(created.id)
            .await
            .expect_err("a deleted label must not be gettable");
        assert!(matches!(err, common::CoreError::NotFound));

        cleanup(&pool, fixture.organization_id, &[fixture.owner_id]).await;
    }

    /// Proves the acceptance criterion end to end: deleting a label removes
    /// it from every task that carried it, via `task_label_links.label_id
    /// ON DELETE CASCADE` — not a soft, application-level cleanup.
    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn delete_task_label_cascades_to_every_task_link() {
        let pool = make_pool().await;
        let fixture = seed_fixture(&pool).await;
        let usecase = make_usecase(pool.clone());

        let task = usecase
            .create_task(create_task_command(&fixture, "Réunion d'équipe"))
            .await
            .unwrap();
        let label_a = usecase
            .create_task_label(create_label_command(fixture.organization_id, "Urgent"))
            .await
            .unwrap();
        let label_b = usecase
            .create_task_label(create_label_command(fixture.organization_id, "Important"))
            .await
            .unwrap();

        let mut patch = PatchTaskCommand::new(task.id);
        patch.label_ids = Some(vec![label_a.id, label_b.id]);
        usecase.patch_task(patch).await.unwrap();

        assert_eq!(count_links_for_label(&pool, label_a.id).await, 1);
        assert_eq!(count_links_for_label(&pool, label_b.id).await, 1);

        usecase.delete_task_label(label_a.id).await.unwrap();

        assert_eq!(
            count_links_for_label(&pool, label_a.id).await,
            0,
            "the link row for the deleted label must be physically gone, not just ignored"
        );
        assert_eq!(
            count_links_for_label(&pool, label_b.id).await,
            1,
            "an unrelated label's link must survive"
        );

        let still_there = usecase.get_task(task.id).await;
        assert!(
            still_there.is_ok(),
            "deleting a label must not touch the task itself"
        );

        cleanup(&pool, fixture.organization_id, &[fixture.owner_id]).await;
    }

    /// Proves `PATCH /tasks`'s `label_ids` contract: the complete
    /// replacement list, never a delta — same semantics as `assignees`.
    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn patch_task_label_ids_replaces_the_complete_set() {
        let pool = make_pool().await;
        let fixture = seed_fixture(&pool).await;
        let usecase = make_usecase(pool.clone());

        let task = usecase
            .create_task(create_task_command(&fixture, "Chantier"))
            .await
            .unwrap();
        let label_a = usecase
            .create_task_label(create_label_command(fixture.organization_id, "Urgent"))
            .await
            .unwrap();
        let label_b = usecase
            .create_task_label(create_label_command(fixture.organization_id, "Important"))
            .await
            .unwrap();
        let label_c = usecase
            .create_task_label(create_label_command(fixture.organization_id, "Externe"))
            .await
            .unwrap();

        let mut first_patch = PatchTaskCommand::new(task.id);
        first_patch.label_ids = Some(vec![label_a.id, label_b.id]);
        usecase.patch_task(first_patch).await.unwrap();

        assert_eq!(count_links_for_label(&pool, label_a.id).await, 1);
        assert_eq!(count_links_for_label(&pool, label_b.id).await, 1);
        assert_eq!(count_links_for_label(&pool, label_c.id).await, 0);

        // A second PATCH names only `label_c` — `label_a`/`label_b` must be
        // dropped, not merged with the new set.
        let mut second_patch = PatchTaskCommand::new(task.id);
        second_patch.label_ids = Some(vec![label_c.id]);
        usecase.patch_task(second_patch).await.unwrap();

        assert_eq!(
            count_links_for_label(&pool, label_a.id).await,
            0,
            "label_ids replaces the set — label_a must be gone"
        );
        assert_eq!(count_links_for_label(&pool, label_b.id).await, 0);
        assert_eq!(count_links_for_label(&pool, label_c.id).await, 1);

        // A patch that never mentions `label_ids` leaves the current set
        // untouched — mirrors `assignees: None`.
        let mut untouched_patch = PatchTaskCommand::new(task.id);
        untouched_patch.title = Some("Chantier renommé".to_owned());
        usecase.patch_task(untouched_patch).await.unwrap();
        assert_eq!(count_links_for_label(&pool, label_c.id).await, 1);

        // An empty `label_ids` clears the set entirely.
        let mut clearing_patch = PatchTaskCommand::new(task.id);
        clearing_patch.label_ids = Some(Vec::new());
        usecase.patch_task(clearing_patch).await.unwrap();
        assert_eq!(
            count_links_for_label(&pool, label_c.id).await,
            0,
            "an empty label_ids must remove every label from the task"
        );

        cleanup(&pool, fixture.organization_id, &[fixture.owner_id]).await;
    }

    /// Proves the acceptance criterion: a freshly created organization
    /// carries its three preset labels, seeded in the same transaction as
    /// its default roles.
    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn create_organization_seeds_the_three_preset_task_labels() {
        let pool = make_pool().await;
        let usecase = make_usecase(pool.clone());

        let owner_uuid = generate_uuid_v7();
        sqlx::query!(
            r#"INSERT INTO users (id, email, username, display_name, sub)
               VALUES ($1, $2, $3, $4, $5)"#,
            owner_uuid,
            format!("owner-{owner_uuid}@example.com"),
            format!("owner-{owner_uuid}"),
            "Owner User",
            owner_uuid.to_string(),
        )
        .execute(&pool)
        .await
        .unwrap();
        let owner_id = UserId(owner_uuid);

        let organization = usecase
            .create_organization(CreateOrganizationCommand {
                name: "Acme Toiture".to_owned(),
                slug: format!("acme-toiture-{owner_uuid}"),
                owner_id,
            })
            .await
            .expect("create_organization must succeed");

        let labels = usecase
            .list_task_labels(organization.id)
            .await
            .expect("list_task_labels must succeed");

        let mut names: Vec<&str> = labels.iter().map(|l| l.name.as_str()).collect();
        names.sort_unstable();
        assert_eq!(labels.len(), 3, "exactly the three presets, no more");
        assert_eq!(names, vec!["Déplacement", "Formation", "Réunion"]);
        assert!(
            labels
                .iter()
                .all(|l| l.color.starts_with('#') && l.color.len() == 7),
            "every preset must carry a well-formed hex color"
        );

        cleanup(&pool, organization.id, &[owner_id]).await;
    }

    // -- list_task_labels_for_tasks: feeds TaskResponse.labels ---------------

    /// Proves the acceptance criterion: a task carrying two labels reports
    /// both, not just one — exercised through the same batched call
    /// `GET /tasks{,/{id}}` and `PATCH /tasks` use to populate
    /// `TaskResponse.labels`.
    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn list_task_labels_for_tasks_returns_every_label_of_a_two_label_task() {
        let pool = make_pool().await;
        let fixture = seed_fixture(&pool).await;
        let usecase = make_usecase(pool.clone());

        let task = usecase
            .create_task(create_task_command(&fixture, "Réunion de chantier"))
            .await
            .unwrap();
        let label_a = usecase
            .create_task_label(create_label_command(fixture.organization_id, "Urgent"))
            .await
            .unwrap();
        let label_b = usecase
            .create_task_label(create_label_command(fixture.organization_id, "Important"))
            .await
            .unwrap();

        let mut patch = PatchTaskCommand::new(task.id);
        patch.label_ids = Some(vec![label_a.id, label_b.id]);
        usecase.patch_task(patch).await.unwrap();

        let mut labels_by_task = usecase
            .list_task_labels_for_tasks(vec![task.id])
            .await
            .expect("list_task_labels_for_tasks must succeed");
        let mut names: Vec<String> = labels_by_task
            .remove(&task.id)
            .expect("the task must have an entry — it carries labels")
            .into_iter()
            .map(|l| l.name)
            .collect();
        names.sort_unstable();

        assert_eq!(names, vec!["Important".to_owned(), "Urgent".to_owned()]);

        cleanup(&pool, fixture.organization_id, &[fixture.owner_id]).await;
    }

    /// A task with no labels is simply absent from the map — mirrors
    /// `TaskRepository::count_children`'s own contract — so `TaskResponse`
    /// building code must fall back to an empty `Vec`, never treat a
    /// missing entry as an error.
    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn list_task_labels_for_tasks_omits_a_labelless_task_from_the_map() {
        let pool = make_pool().await;
        let fixture = seed_fixture(&pool).await;
        let usecase = make_usecase(pool.clone());

        let task = usecase
            .create_task(create_task_command(&fixture, "Sans étiquette"))
            .await
            .unwrap();

        let labels_by_task = usecase
            .list_task_labels_for_tasks(vec![task.id])
            .await
            .unwrap();

        assert!(
            !labels_by_task.contains_key(&task.id),
            "a task with no labels must be absent from the map, not mapped to an empty Vec"
        );

        cleanup(&pool, fixture.organization_id, &[fixture.owner_id]).await;
    }

    /// Proves `GET /tasks` does not degenerate into one label query per
    /// task: three tasks, with disjoint label sets (including one task with
    /// none), are all resolved correctly by a *single* call to
    /// `list_task_labels_for_tasks` carrying every id at once — exactly what
    /// `handlers-planning/src/task/list.rs` does for a whole page. The
    /// underlying `PgTaskLabelRepository::list_labels_for_tasks` issues one
    /// SQL statement (`... WHERE task_id = ANY($1)`) regardless of how many
    /// ids it's given — there is no loop over `task_ids` anywhere on this
    /// path — so the query count this test's single call makes is constant,
    /// not a function of the number of tasks in the page.
    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn list_task_labels_for_tasks_batches_an_entire_page_in_one_call() {
        let pool = make_pool().await;
        let fixture = seed_fixture(&pool).await;
        let usecase = make_usecase(pool.clone());

        let shared_label = usecase
            .create_task_label(create_label_command(fixture.organization_id, "Urgent"))
            .await
            .unwrap();
        let solo_label = usecase
            .create_task_label(create_label_command(fixture.organization_id, "Important"))
            .await
            .unwrap();

        let task_a = usecase
            .create_task(create_task_command(&fixture, "Tâche A"))
            .await
            .unwrap();
        let task_b = usecase
            .create_task(create_task_command(&fixture, "Tâche B"))
            .await
            .unwrap();
        let task_c = usecase
            .create_task(create_task_command(&fixture, "Tâche C — sans étiquette"))
            .await
            .unwrap();

        let mut patch_a = PatchTaskCommand::new(task_a.id);
        patch_a.label_ids = Some(vec![shared_label.id]);
        usecase.patch_task(patch_a).await.unwrap();

        let mut patch_b = PatchTaskCommand::new(task_b.id);
        patch_b.label_ids = Some(vec![shared_label.id, solo_label.id]);
        usecase.patch_task(patch_b).await.unwrap();
        // task_c is left unpatched — no labels.

        // One call, every task id at once — the shape `GET /tasks` uses for
        // a whole page.
        let mut labels_by_task = usecase
            .list_task_labels_for_tasks(vec![task_a.id, task_b.id, task_c.id])
            .await
            .expect("a single batched call must resolve every task's labels");

        let names_a: Vec<String> = labels_by_task
            .remove(&task_a.id)
            .unwrap()
            .into_iter()
            .map(|l| l.name)
            .collect();
        assert_eq!(names_a, vec!["Urgent".to_owned()]);

        let mut names_b: Vec<String> = labels_by_task
            .remove(&task_b.id)
            .unwrap()
            .into_iter()
            .map(|l| l.name)
            .collect();
        names_b.sort_unstable();
        assert_eq!(names_b, vec!["Important".to_owned(), "Urgent".to_owned()]);

        assert!(
            !labels_by_task.contains_key(&task_c.id),
            "task_c carries no labels and must be absent from the map"
        );

        cleanup(&pool, fixture.organization_id, &[fixture.owner_id]).await;
    }
}
