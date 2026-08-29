#[cfg(test)]
#[allow(clippy::module_inception)]
mod tests {

    use chrono::{DateTime, Duration, Utc};
    use common::{CoreError, OrganizationId, UserId, generate_uuid_v7};
    use sqlx::PgPool;

    use crate::application::test_support::{dev_pool, purge};
    use crate::application::{MestierUseCase, default_authorizer};
    use crate::domain::task_comment::commands::{
        CreateTaskCommentCommand, UpdateTaskCommentCommand,
    };
    use crate::infrastructure::realtime::EventHub;
    use crate::{CreateTaskCommand, TaskCommentId, TaskId};

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

    async fn seed_user(pool: &PgPool, label: &str) -> UserId {
        let id = generate_uuid_v7();
        sqlx::query!(
            r#"INSERT INTO users (id, email, username, display_name, sub)
               VALUES ($1, $2, $3, $4, $5)"#,
            id,
            format!("{label}-{id}@example.com"),
            format!("{label}-{id}"),
            format!("{label} User"),
            format!("sub-{label}-{id}"),
        )
        .execute(pool)
        .await
        .unwrap();

        UserId(id)
    }

    /// Seeds an owner user and an organization — the minimal graph a task
    /// comment needs to exist. Unlike `task`'s own fixture, no customer is
    /// seeded: a comment thread never depends on a chantier.
    async fn seed_fixture(pool: &PgPool) -> Fixture {
        let owner_id = seed_user(pool, "owner").await;

        let org_id = generate_uuid_v7();
        sqlx::query!(
            r#"INSERT INTO organizations (id, name, slug, owner_id)
               VALUES ($1, $2, $3, $4)"#,
            org_id,
            "Test Org",
            format!("test-org-{org_id}"),
            owner_id.0,
        )
        .execute(pool)
        .await
        .unwrap();

        Fixture {
            organization_id: OrganizationId(org_id),
            owner_id,
        }
    }

    /// Removes everything seeded under `organization_id`: comments cascade
    /// from `tasks` themselves via `ON DELETE CASCADE`, so only `tasks`,
    /// `employees`, the organization and its loose users need explicit
    /// cleanup.
    async fn cleanup(pool: &PgPool, organization_id: OrganizationId, user_ids: &[UserId]) {
        purge(
            pool,
            "DELETE FROM task_comments WHERE org_id = $1",
            organization_id.0,
        )
        .await;
        purge(
            pool,
            "DELETE FROM task_assignments WHERE org_id = $1",
            organization_id.0,
        )
        .await;
        purge(
            pool,
            "DELETE FROM employees WHERE org_id = $1",
            organization_id.0,
        )
        .await;
        purge(
            pool,
            "DELETE FROM tasks WHERE org_id = $1",
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

    fn create_task_command(fixture: &Fixture, title: &str) -> CreateTaskCommand {
        let now = Utc::now();
        CreateTaskCommand {
            actor: authz::Subject::system(),
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
            project_id: None,
            expenses_cents: 0,
            expenses_label: None,
        }
    }

    fn create_comment_command(
        fixture: &Fixture,
        task_id: TaskId,
        body: &str,
    ) -> CreateTaskCommentCommand {
        CreateTaskCommentCommand {
            actor: authz::Subject::system(),
            organization_id: fixture.organization_id,
            task_id,
            author_user_id: fixture.owner_id,
            body: body.to_owned(),
        }
    }

    async fn fetch_comment_row(
        pool: &PgPool,
        id: TaskCommentId,
    ) -> Option<(String, Option<DateTime<Utc>>)> {
        sqlx::query!(
            r#"SELECT body, deleted_at FROM task_comments WHERE id = $1"#,
            id.0,
        )
        .fetch_optional(pool)
        .await
        .unwrap()
        .map(|row| (row.body, row.deleted_at))
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn create_task_comment_persists_and_is_retrievable() {
        let pool = make_pool().await;
        let fixture = seed_fixture(&pool).await;
        let usecase = make_usecase(pool.clone());

        let task = usecase
            .create_task(create_task_command(&fixture, "Réunion de chantier"))
            .await
            .unwrap();

        let created = usecase
            .create_task_comment(create_comment_command(&fixture, task.id, "Salut l'équipe"))
            .await
            .expect("create_task_comment must succeed");

        assert_eq!(created.body, "Salut l'équipe");
        assert_eq!(created.author_user_id, fixture.owner_id);
        assert_eq!(created.task_id, task.id);

        let fetched = usecase
            .get_task_comment(created.id)
            .await
            .expect("get_task_comment must succeed");
        assert_eq!(fetched.id, created.id);

        cleanup(&pool, fixture.organization_id, &[fixture.owner_id]).await;
    }

    /// Proves the acceptance criterion at the database level — bypassing
    /// the domain's own `validate_body` guard entirely, via a raw insert —
    /// so `chk_task_comments_body_not_blank` is shown to stand on its own,
    /// not merely mirror the application-level check.
    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn a_blank_body_is_rejected_by_the_database_check_constraint() {
        let pool = make_pool().await;
        let fixture = seed_fixture(&pool).await;
        let usecase = make_usecase(pool.clone());

        let task = usecase
            .create_task(create_task_command(&fixture, "Réunion"))
            .await
            .unwrap();

        let result = sqlx::query!(
            r#"INSERT INTO task_comments (id, org_id, task_id, author_user_id, body)
               VALUES ($1, $2, $3, $4, $5)"#,
            generate_uuid_v7(),
            fixture.organization_id.0,
            task.id.0,
            fixture.owner_id.0,
            "   ",
        )
        .execute(&pool)
        .await;

        assert!(
            result.is_err(),
            "a blank body must be rejected by the database's own CHECK constraint"
        );

        cleanup(&pool, fixture.organization_id, &[fixture.owner_id]).await;
    }

    /// Proves the acceptance criterion: `author_user_id` references
    /// `users(id)`, never `employees(id)` — an employee's own id, though a
    /// real row in this organization, cannot satisfy the foreign key. An
    /// employee without a user account is therefore structurally incapable
    /// of authoring a comment.
    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn a_seat_without_an_account_cannot_be_an_author() {
        let pool = make_pool().await;
        let fixture = seed_fixture(&pool).await;
        let usecase = make_usecase(pool.clone());

        let task = usecase
            .create_task(create_task_command(&fixture, "Réunion"))
            .await
            .unwrap();

        // A free seat: nobody holds it, so nothing about it can author a
        // comment. Seeded directly — creating one is the members API's job,
        // not this test's subject.
        let member_id = generate_uuid_v7();
        sqlx::query!(
            r#"INSERT INTO organization_members (id, organization_id, last_name)
               VALUES ($1, $2, $3)"#,
            member_id,
            fixture.organization_id.0,
            "Sans compte",
        )
        .execute(&pool)
        .await
        .expect("seeding a free seat must succeed");

        let result = sqlx::query!(
            r#"INSERT INTO task_comments (id, org_id, task_id, author_user_id, body)
               VALUES ($1, $2, $3, $4, $5)"#,
            generate_uuid_v7(),
            fixture.organization_id.0,
            task.id.0,
            member_id,
            "Je commente en tant qu'employé",
        )
        .execute(&pool)
        .await;

        assert!(
            result.is_err(),
            "an employee id must not satisfy the author_user_id foreign key to users(id)"
        );

        cleanup(&pool, fixture.organization_id, &[fixture.owner_id]).await;
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn list_task_comments_returns_the_thread_in_chronological_order() {
        let pool = make_pool().await;
        let fixture = seed_fixture(&pool).await;
        let usecase = make_usecase(pool.clone());

        let task = usecase
            .create_task(create_task_command(&fixture, "Réunion"))
            .await
            .unwrap();

        usecase
            .create_task_comment(create_comment_command(&fixture, task.id, "Premier"))
            .await
            .unwrap();
        usecase
            .create_task_comment(create_comment_command(&fixture, task.id, "Deuxième"))
            .await
            .unwrap();
        usecase
            .create_task_comment(create_comment_command(&fixture, task.id, "Troisième"))
            .await
            .unwrap();

        let (items, total) = usecase.list_task_comments(task.id, 20, 0).await.unwrap();

        assert_eq!(total, 3);
        assert_eq!(
            items.iter().map(|c| c.body.as_str()).collect::<Vec<_>>(),
            vec!["Premier", "Deuxième", "Troisième"]
        );

        cleanup(&pool, fixture.organization_id, &[fixture.owner_id]).await;
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn list_task_comments_returns_the_correct_page() {
        let pool = make_pool().await;
        let fixture = seed_fixture(&pool).await;
        let usecase = make_usecase(pool.clone());

        let task = usecase
            .create_task(create_task_command(&fixture, "Réunion"))
            .await
            .unwrap();

        for body in ["Un", "Deux", "Trois", "Quatre", "Cinq"] {
            usecase
                .create_task_comment(create_comment_command(&fixture, task.id, body))
                .await
                .unwrap();
        }

        let (first_page, total) = usecase.list_task_comments(task.id, 2, 0).await.unwrap();
        assert_eq!(total, 5);
        assert_eq!(
            first_page
                .iter()
                .map(|c| c.body.as_str())
                .collect::<Vec<_>>(),
            vec!["Un", "Deux"]
        );

        let (second_page, _) = usecase.list_task_comments(task.id, 2, 2).await.unwrap();
        assert_eq!(
            second_page
                .iter()
                .map(|c| c.body.as_str())
                .collect::<Vec<_>>(),
            vec!["Trois", "Quatre"]
        );

        let (last_page, _) = usecase.list_task_comments(task.id, 2, 4).await.unwrap();
        assert_eq!(
            last_page
                .iter()
                .map(|c| c.body.as_str())
                .collect::<Vec<_>>(),
            vec!["Cinq"]
        );

        cleanup(&pool, fixture.organization_id, &[fixture.owner_id]).await;
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn update_task_comment_by_its_author_succeeds() {
        let pool = make_pool().await;
        let fixture = seed_fixture(&pool).await;
        let usecase = make_usecase(pool.clone());

        let task = usecase
            .create_task(create_task_command(&fixture, "Réunion"))
            .await
            .unwrap();
        let created = usecase
            .create_task_comment(create_comment_command(&fixture, task.id, "Original"))
            .await
            .unwrap();

        let updated = usecase
            .update_task_comment(UpdateTaskCommentCommand {
                actor: authz::Subject::system(),
                id: created.id,
                acting_user_id: fixture.owner_id,
                body: "Édité".to_owned(),
            })
            .await
            .expect("the author must be able to edit their own comment");

        assert_eq!(updated.body, "Édité");
        assert!(updated.updated_at >= created.updated_at);

        cleanup(&pool, fixture.organization_id, &[fixture.owner_id]).await;
    }

    /// Proves the acceptance criterion end to end, through the real
    /// transactional path rather than a domain mock: another user editing
    /// someone else's comment gets `Forbidden`, never a silent success.
    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn update_task_comment_by_someone_else_is_forbidden() {
        let pool = make_pool().await;
        let fixture = seed_fixture(&pool).await;
        let other_user_id = seed_user(&pool, "intruder").await;
        let usecase = make_usecase(pool.clone());

        let task = usecase
            .create_task(create_task_command(&fixture, "Réunion"))
            .await
            .unwrap();
        let created = usecase
            .create_task_comment(create_comment_command(&fixture, task.id, "Original"))
            .await
            .unwrap();

        let err = usecase
            .update_task_comment(UpdateTaskCommentCommand {
                actor: authz::Subject::system(),
                id: created.id,
                acting_user_id: other_user_id,
                body: "Je réécris ton message".to_owned(),
            })
            .await
            .expect_err("a non-author must not be able to edit the comment");

        assert!(matches!(err, CoreError::Forbidden { .. }));

        let (row_body, _) = fetch_comment_row(&pool, created.id).await.unwrap();
        assert_eq!(
            row_body, "Original",
            "a rejected PATCH must not have mutated the row"
        );

        cleanup(
            &pool,
            fixture.organization_id,
            &[fixture.owner_id, other_user_id],
        )
        .await;
    }

    /// Same security rule as the `PATCH` test above, for `DELETE`.
    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn delete_task_comment_by_someone_else_is_forbidden() {
        let pool = make_pool().await;
        let fixture = seed_fixture(&pool).await;
        let other_user_id = seed_user(&pool, "intruder").await;
        let usecase = make_usecase(pool.clone());

        let task = usecase
            .create_task(create_task_command(&fixture, "Réunion"))
            .await
            .unwrap();
        let created = usecase
            .create_task_comment(create_comment_command(&fixture, task.id, "Original"))
            .await
            .unwrap();

        let err = usecase
            .delete_task_comment(authz::Subject::system(), created.id, other_user_id)
            .await
            .expect_err("a non-author must not be able to delete the comment");

        assert!(matches!(err, CoreError::Forbidden { .. }));

        let (_, deleted_at) = fetch_comment_row(&pool, created.id).await.unwrap();
        assert!(
            deleted_at.is_none(),
            "a rejected DELETE must not have soft-deleted the row"
        );

        cleanup(
            &pool,
            fixture.organization_id,
            &[fixture.owner_id, other_user_id],
        )
        .await;
    }

    /// Proves the acceptance criterion: a deleted comment disappears from
    /// the thread (`get_task_comment`/`list_task_comments` exclude it), but
    /// the row survives in the table with `deleted_at` set — unlike
    /// `task_label_links`'s physical delete.
    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn delete_task_comment_removes_it_from_the_thread_but_the_row_survives() {
        let pool = make_pool().await;
        let fixture = seed_fixture(&pool).await;
        let usecase = make_usecase(pool.clone());

        let task = usecase
            .create_task(create_task_command(&fixture, "Réunion"))
            .await
            .unwrap();
        let created = usecase
            .create_task_comment(create_comment_command(&fixture, task.id, "À retirer"))
            .await
            .unwrap();

        usecase
            .delete_task_comment(authz::Subject::system(), created.id, fixture.owner_id)
            .await
            .expect("the author must be able to delete their own comment");

        let err = usecase
            .get_task_comment(created.id)
            .await
            .expect_err("a deleted comment must not be gettable");
        assert!(matches!(err, CoreError::NotFound));

        let (items, total) = usecase.list_task_comments(task.id, 20, 0).await.unwrap();
        assert_eq!(
            total, 0,
            "a deleted comment must not count towards the thread's total"
        );
        assert!(
            items.is_empty(),
            "a deleted comment must not appear in the thread"
        );

        let row = fetch_comment_row(&pool, created.id).await;
        assert!(
            row.is_some(),
            "the row must still exist in the database after a soft delete"
        );
        let (_, deleted_at) = row.unwrap();
        assert!(deleted_at.is_some(), "deleted_at must be set");

        cleanup(&pool, fixture.organization_id, &[fixture.owner_id]).await;
    }

    /// Proves the acceptance criterion via the live `ON DELETE CASCADE`
    /// itself — a *physical* delete of the task row, not the application's
    /// own `soft_delete_task` (which never removes a row) — rather than
    /// trusting the migration file.
    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn deleting_a_task_cascades_to_its_comments() {
        let pool = make_pool().await;
        let fixture = seed_fixture(&pool).await;
        let usecase = make_usecase(pool.clone());

        let task = usecase
            .create_task(create_task_command(&fixture, "Réunion"))
            .await
            .unwrap();
        let created = usecase
            .create_task_comment(create_comment_command(&fixture, task.id, "Va disparaître"))
            .await
            .unwrap();

        sqlx::query!("DELETE FROM tasks WHERE id = $1", task.id.0)
            .execute(&pool)
            .await
            .expect("physically deleting the task must succeed");

        let row = fetch_comment_row(&pool, created.id).await;
        assert!(
            row.is_none(),
            "ON DELETE CASCADE must have physically removed the comment along with its task"
        );

        cleanup(&pool, fixture.organization_id, &[fixture.owner_id]).await;
    }

    /// Proves `GET .../comments`'s author-batching contract: two comments
    /// by two different authors are both resolved by a *single* call to
    /// `list_task_comment_authors`, carrying every distinct author id at
    /// once — exactly what `handlers-planning/src/task_comment/list.rs`
    /// does for a whole page. `UserRepository::list_by_ids`'s own
    /// implementation issues one `WHERE id = ANY($1)` statement regardless
    /// of how many ids it is given, so this call's query count does not
    /// grow with the number of comments in the page.
    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn list_task_comment_authors_batches_every_distinct_author_in_one_call() {
        let pool = make_pool().await;
        let fixture = seed_fixture(&pool).await;
        let second_author_id = seed_user(&pool, "second-author").await;
        let usecase = make_usecase(pool.clone());

        let task = usecase
            .create_task(create_task_command(&fixture, "Réunion"))
            .await
            .unwrap();
        usecase
            .create_task_comment(create_comment_command(&fixture, task.id, "Du propriétaire"))
            .await
            .unwrap();
        usecase
            .create_task_comment(CreateTaskCommentCommand {
                actor: authz::Subject::system(),
                organization_id: fixture.organization_id,
                task_id: task.id,
                author_user_id: second_author_id,
                body: "Du second auteur".to_owned(),
            })
            .await
            .unwrap();

        let (comments, _) = usecase.list_task_comments(task.id, 20, 0).await.unwrap();
        let author_ids: Vec<UserId> = comments.iter().map(|c| c.author_user_id).collect();

        let authors = usecase
            .list_task_comment_authors(author_ids)
            .await
            .expect("a single batched call must resolve every comment's author");

        assert_eq!(authors.len(), 2);
        assert!(authors.contains_key(&fixture.owner_id));
        assert!(authors.contains_key(&second_author_id));

        cleanup(
            &pool,
            fixture.organization_id,
            &[fixture.owner_id, second_author_id],
        )
        .await;
    }
}
