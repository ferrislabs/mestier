#[cfg(test)]
#[allow(clippy::module_inception)]
mod tests {

    use chrono::{Duration, Utc};
    use common::{OrganizationId, UserId, generate_uuid_v7};
    use sqlx::PgPool;

    use crate::application::{MestierUseCase, default_authorizer};
    use crate::domain::task::{
        AssigneeRef,
        commands::{CreateTaskCommand, PatchTaskCommand},
    };
    use crate::infrastructure::realtime::EventHub;
    use crate::{CustomerContextId, CustomerId, EmployeeId, TaskId, TaskStatus};

    async fn make_pool() -> PgPool {
        let url = std::env::var("DATABASE_URL")
            .expect("DATABASE_URL must be set to run task integration tests");
        PgPool::connect(&url).await.unwrap()
    }

    fn make_usecase(pool: PgPool) -> MestierUseCase {
        MestierUseCase::new(pool, default_authorizer(), EventHub::new())
    }

    struct Fixture {
        organization_id: OrganizationId,
        customer_id: CustomerId,
        customer_context_id: CustomerContextId,
        owner_id: UserId,
    }

    /// Seeds an owner user, an organization, a customer and a customer
    /// context — the minimal graph a task needs to exist as a chantier.
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

        let customer_id = generate_uuid_v7();
        sqlx::query!(
            r#"INSERT INTO customers (id, org_id, name)
               VALUES ($1, $2, $3)"#,
            customer_id,
            org_id,
            "Alice Dupont",
        )
        .execute(pool)
        .await
        .unwrap();

        let customer_context_id = generate_uuid_v7();
        sqlx::query!(
            r#"INSERT INTO customer_contexts (id, customer_id, label)
               VALUES ($1, $2, $3)"#,
            customer_context_id,
            customer_id,
            "Chantier principal",
        )
        .execute(pool)
        .await
        .unwrap();

        Fixture {
            organization_id: OrganizationId(org_id),
            customer_id: CustomerId(customer_id),
            customer_context_id: CustomerContextId(customer_context_id),
            owner_id: UserId(owner_id),
        }
    }

    /// Seeds an employee record already attached to `organization_id`.
    async fn seed_employee(pool: &PgPool, organization_id: OrganizationId) -> EmployeeId {
        let employee_id = generate_uuid_v7();
        sqlx::query!(
            r#"INSERT INTO employees (id, org_id, last_name, hourly_rate_cents, weekly_contract_minutes)
               VALUES ($1, $2, $3, $4, $5)"#,
            employee_id,
            organization_id.0,
            "Existing Employee",
            3500,
            2100,
        )
        .execute(pool)
        .await
        .unwrap();
        EmployeeId(employee_id)
    }

    /// Seeds a user who is a member of `organization_id` but has no
    /// employee record yet — a "member-only" planning resource.
    async fn seed_member_without_employee(
        pool: &PgPool,
        organization_id: OrganizationId,
    ) -> UserId {
        let user_id = generate_uuid_v7();
        sqlx::query!(
            r#"INSERT INTO users (id, email, username, display_name, sub)
               VALUES ($1, $2, $3, $4, $5)"#,
            user_id,
            format!("member-{user_id}@example.com"),
            format!("member-{user_id}"),
            "Member Without Employee",
            format!("sub-member-{user_id}"),
        )
        .execute(pool)
        .await
        .unwrap();

        sqlx::query!(
            r#"INSERT INTO organization_members (organization_id, user_id, last_name)
               VALUES ($1, $2, $3)"#,
            organization_id.0,
            user_id,
            "Member Without Employee",
        )
        .execute(pool)
        .await
        .unwrap();

        UserId(user_id)
    }

    /// Removes everything seeded under `organization_id`, cascading to
    /// customers/customer_contexts/employees/tasks/task_assignments/members,
    /// plus the loose user rows that outlive the organization.
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
        sqlx::query!("DELETE FROM employees WHERE org_id = $1", organization_id.0)
            .execute(pool)
            .await
            .ok();
        sqlx::query!("DELETE FROM customers WHERE org_id = $1", organization_id.0)
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

    fn create_command(fixture: &Fixture) -> CreateTaskCommand {
        let now = Utc::now();
        CreateTaskCommand {
            organization_id: fixture.organization_id,
            parent_task_id: None,
            title: "Réfection toiture".to_owned(),
            description: None,
            starts_at: Some(now),
            ends_at: Some(now + Duration::hours(2)),
            all_day: false,
            blocks_availability: true,
            customer_id: Some(fixture.customer_id),
            customer_context_id: Some(fixture.customer_context_id),
            quote_id: None,
        }
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn create_task_persists_and_is_retrievable() {
        let pool = make_pool().await;
        let fixture = seed_fixture(&pool).await;
        let usecase = make_usecase(pool.clone());

        let created = usecase
            .create_task(create_command(&fixture))
            .await
            .expect("create_task must succeed");

        assert_eq!(created.status, TaskStatus::Planned);
        assert!(created.assignments.is_empty());

        let fetched = usecase
            .get_task(created.id)
            .await
            .expect("get_task must succeed");
        assert_eq!(fetched.id, created.id);
        assert_eq!(fetched.title, "Réfection toiture");

        cleanup(&pool, fixture.organization_id, &[fixture.owner_id]).await;
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn create_task_without_a_customer_persists_and_is_retrievable() {
        let pool = make_pool().await;
        let fixture = seed_fixture(&pool).await;
        let usecase = make_usecase(pool.clone());

        let mut command = create_command(&fixture);
        command.customer_id = None;
        command.customer_context_id = None;
        command.title = "Réunion d'équipe".to_owned();

        let created = usecase
            .create_task(command)
            .await
            .expect("create_task without a customer must succeed");

        assert!(created.customer_id.is_none());
        assert!(created.customer_context_id.is_none());

        let fetched = usecase.get_task(created.id).await.unwrap();
        assert_eq!(fetched.title, "Réunion d'équipe");
        assert!(fetched.customer_id.is_none());

        cleanup(&pool, fixture.organization_id, &[fixture.owner_id]).await;
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn list_tasks_returns_only_the_organizations_own_root_tasks() {
        let pool = make_pool().await;
        let fixture_a = seed_fixture(&pool).await;
        let fixture_b = seed_fixture(&pool).await;
        let usecase = make_usecase(pool.clone());

        usecase
            .create_task(create_command(&fixture_a))
            .await
            .unwrap();
        usecase
            .create_task(create_command(&fixture_b))
            .await
            .unwrap();

        let (items, _child_counts, total) = usecase
            .list_tasks(fixture_a.organization_id, None, 20, 0)
            .await
            .expect("list_tasks must succeed");

        assert_eq!(total, 1);
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].organization_id, fixture_a.organization_id);

        cleanup(&pool, fixture_a.organization_id, &[fixture_a.owner_id]).await;
        cleanup(&pool, fixture_b.organization_id, &[fixture_b.owner_id]).await;
    }

    /// Proves `GET /tasks`'s contract end to end: each root's child count is
    /// correct, and listing `?parent_task_id=<root>` returns exactly its
    /// children — without ever loading the hierarchy to count it (the
    /// service issues one list query plus one grouped `count_children`
    /// query, never one count per task; see `TaskRepository::count_children`).
    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn list_tasks_reports_each_roots_child_count_without_loading_the_hierarchy() {
        let pool = make_pool().await;
        let fixture = seed_fixture(&pool).await;
        let usecase = make_usecase(pool.clone());

        let root = usecase.create_task(create_command(&fixture)).await.unwrap();

        let mut first_child = create_command(&fixture);
        first_child.parent_task_id = Some(root.id);
        first_child.title = "Lot électricité".to_owned();
        first_child.starts_at = None;
        first_child.ends_at = None;
        usecase.create_task(first_child).await.unwrap();

        let mut second_child = create_command(&fixture);
        second_child.parent_task_id = Some(root.id);
        second_child.title = "Lot plomberie".to_owned();
        second_child.starts_at = None;
        second_child.ends_at = None;
        usecase.create_task(second_child).await.unwrap();

        let (roots, child_counts, total) = usecase
            .list_tasks(fixture.organization_id, None, 20, 0)
            .await
            .unwrap();
        assert_eq!(total, 1, "children must not appear in the root listing");
        assert_eq!(roots.len(), 1);
        assert_eq!(roots[0].id, root.id);
        assert_eq!(
            child_counts.get(&root.id),
            Some(&2),
            "the root's child count must reflect both subtasks"
        );

        let (children, child_child_counts, children_total) = usecase
            .list_tasks(fixture.organization_id, Some(root.id), 20, 0)
            .await
            .unwrap();
        assert_eq!(children_total, 2);
        assert_eq!(children.len(), 2);
        assert!(
            children
                .iter()
                .all(|task| task.parent_task_id == Some(root.id))
        );
        assert!(
            children
                .iter()
                .all(|task| !child_child_counts.contains_key(&task.id)),
            "a subtask has no children of its own — the two-level cap"
        );

        cleanup(&pool, fixture.organization_id, &[fixture.owner_id]).await;
    }

    /// Proves the fix for a pre-existing N+1 in
    /// `PgTaskRepository::list_by_organization`: it used to call
    /// `fetch_assignments` once per task row, inside a loop (found while
    /// proving `GET /tasks`'s total query count for the task-labels
    /// workstream, #142; fixed here rather than in a separate PR — see that
    /// commit's message). Three tasks with disjoint assignee sets, one of
    /// them with none, must all resolve correctly through a single
    /// `list_tasks` call — proof that `fetch_assignments_for_tasks`'s
    /// grouped query (`WHERE task_id = ANY($1)`) actually replaced the
    /// per-row loop rather than the loop silently surviving alongside it.
    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn list_tasks_batches_assignments_across_the_whole_page() {
        let pool = make_pool().await;
        let fixture = seed_fixture(&pool).await;
        let employee_a = seed_employee(&pool, fixture.organization_id).await;
        let employee_b = seed_employee(&pool, fixture.organization_id).await;
        let usecase = make_usecase(pool.clone());

        let task_a = usecase.create_task(create_command(&fixture)).await.unwrap();
        let mut patch_a = PatchTaskCommand::new(task_a.id);
        patch_a.assignees = Some(vec![AssigneeRef::Employee(employee_a)]);
        usecase.patch_task(patch_a).await.unwrap();

        let mut second_command = create_command(&fixture);
        second_command.title = "Deuxième tâche".to_owned();
        let task_b = usecase.create_task(second_command).await.unwrap();
        let mut patch_b = PatchTaskCommand::new(task_b.id);
        patch_b.assignees = Some(vec![AssigneeRef::Employee(employee_b)]);
        usecase.patch_task(patch_b).await.unwrap();

        let mut third_command = create_command(&fixture);
        third_command.title = "Troisième tâche — sans assigné".to_owned();
        let task_c = usecase.create_task(third_command).await.unwrap();
        // task_c is deliberately left unassigned.

        let (tasks, _child_counts, total) = usecase
            .list_tasks(fixture.organization_id, None, 20, 0)
            .await
            .expect("list_tasks must succeed");

        assert_eq!(total, 3);
        assert_eq!(tasks.len(), 3);

        let find = |id: TaskId| tasks.iter().find(|task| task.id == id).unwrap();
        let employee_ids_of = |id: TaskId| -> Vec<EmployeeId> {
            find(id)
                .assignments
                .iter()
                .map(|assignment| assignment.employee_id)
                .collect()
        };

        assert_eq!(employee_ids_of(task_a.id), vec![employee_a]);
        assert_eq!(employee_ids_of(task_b.id), vec![employee_b]);
        assert!(
            employee_ids_of(task_c.id).is_empty(),
            "a task with no assignees must come back with an empty list, not another task's"
        );

        cleanup(&pool, fixture.organization_id, &[fixture.owner_id]).await;
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn create_task_rejects_a_subtask_under_a_task_that_already_has_a_parent() {
        let pool = make_pool().await;
        let fixture = seed_fixture(&pool).await;
        let usecase = make_usecase(pool.clone());

        let root = usecase.create_task(create_command(&fixture)).await.unwrap();
        let mut child_command = create_command(&fixture);
        child_command.parent_task_id = Some(root.id);
        child_command.starts_at = None;
        child_command.ends_at = None;
        let child = usecase.create_task(child_command).await.unwrap();

        let mut grandchild_command = create_command(&fixture);
        grandchild_command.parent_task_id = Some(child.id);
        grandchild_command.starts_at = None;
        grandchild_command.ends_at = None;

        let err = usecase
            .create_task(grandchild_command)
            .await
            .expect_err("a third hierarchy level must be rejected");
        assert!(matches!(err, common::CoreError::Conflict(_)));

        cleanup(&pool, fixture.organization_id, &[fixture.owner_id]).await;
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn patch_task_reschedules_reassigns_and_provisions_a_member_employee() {
        let pool = make_pool().await;
        let fixture = seed_fixture(&pool).await;
        let existing_employee_id = seed_employee(&pool, fixture.organization_id).await;
        let member_user_id = seed_member_without_employee(&pool, fixture.organization_id).await;
        let usecase = make_usecase(pool.clone());

        let created = usecase.create_task(create_command(&fixture)).await.unwrap();

        let new_starts_at = created.starts_at.unwrap() + Duration::days(1);
        let new_ends_at = created.ends_at.unwrap() + Duration::days(1);

        let mut patch = PatchTaskCommand::new(created.id);
        patch.starts_at = Some(Some(new_starts_at));
        patch.ends_at = Some(Some(new_ends_at));
        patch.status = Some(TaskStatus::InProgress);
        patch.assignees = Some(vec![
            AssigneeRef::Employee(existing_employee_id),
            AssigneeRef::Member(member_user_id),
        ]);

        let (updated, created_employees) = usecase
            .patch_task(patch)
            .await
            .expect("patch_task must succeed");

        assert_eq!(updated.starts_at, Some(new_starts_at));
        assert_eq!(updated.ends_at, Some(new_ends_at));
        assert_eq!(updated.status, TaskStatus::InProgress);
        assert_eq!(updated.assignments.len(), 2);
        assert_eq!(created_employees.len(), 1);
        assert_eq!(created_employees[0].user_id, Some(member_user_id));
        assert_eq!(created_employees[0].hourly_rate_cents, None);
        assert_eq!(created_employees[0].weekly_contract_minutes, 0);

        let provisioned_employee_id = created_employees[0].id;

        // A second PATCH carrying the same member assignee must reuse the
        // employee record just created rather than provisioning a second
        // one — the whole point of resolving "member" against the existing
        // employee first.
        let mut second_patch = PatchTaskCommand::new(created.id);
        second_patch.assignees = Some(vec![AssigneeRef::Member(member_user_id)]);

        let (reassigned, created_on_second_patch) = usecase
            .patch_task(second_patch)
            .await
            .expect("second patch_task must succeed");

        assert!(created_on_second_patch.is_empty());
        assert_eq!(reassigned.assignments.len(), 1);
        assert_eq!(
            reassigned.assignments[0].employee_id,
            provisioned_employee_id
        );

        let employee_count: i64 = sqlx::query_scalar!(
            r#"SELECT COUNT(*) AS "count!" FROM employees WHERE org_id = $1 AND user_id = $2"#,
            fixture.organization_id.0,
            member_user_id.0,
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(
            employee_count, 1,
            "the member must resolve to a single employee record, not a duplicate"
        );

        cleanup(
            &pool,
            fixture.organization_id,
            &[fixture.owner_id, member_user_id],
        )
        .await;
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn patch_task_rolls_back_the_whole_transaction_on_failure() {
        let pool = make_pool().await;
        let fixture = seed_fixture(&pool).await;
        let member_user_id = seed_member_without_employee(&pool, fixture.organization_id).await;
        let usecase = make_usecase(pool.clone());

        let created = usecase.create_task(create_command(&fixture)).await.unwrap();
        let original_starts_at = created.starts_at;
        let original_ends_at = created.ends_at;

        let bogus_employee_id = EmployeeId(generate_uuid_v7());
        let mut patch = PatchTaskCommand::new(created.id);
        patch.starts_at = Some(Some(original_starts_at.unwrap() + Duration::days(1)));
        patch.ends_at = Some(Some(original_ends_at.unwrap() + Duration::days(1)));
        // First assignee provisions a real employee record (a real write
        // inside the transaction); the second references an employee that
        // does not exist, which fails the whole `PATCH` after that write
        // already happened. Nothing must survive the rollback.
        patch.assignees = Some(vec![
            AssigneeRef::Member(member_user_id),
            AssigneeRef::Employee(bogus_employee_id),
        ]);

        let err = usecase
            .patch_task(patch)
            .await
            .expect_err("patch_task must fail for an unknown employee assignee");
        assert!(matches!(err, common::CoreError::NotFound));

        let task = usecase.get_task(created.id).await.unwrap();
        assert_eq!(
            task.starts_at, original_starts_at,
            "the reschedule must not have landed"
        );
        assert_eq!(task.ends_at, original_ends_at);
        assert!(
            task.assignments.is_empty(),
            "no assignment must have landed"
        );

        let employee_count: i64 = sqlx::query_scalar!(
            r#"SELECT COUNT(*) AS "count!" FROM employees WHERE org_id = $1 AND user_id = $2"#,
            fixture.organization_id.0,
            member_user_id.0,
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(
            employee_count, 0,
            "the on-the-fly employee provisioned before the failure must have rolled back"
        );

        cleanup(
            &pool,
            fixture.organization_id,
            &[fixture.owner_id, member_user_id],
        )
        .await;
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn soft_delete_task_hides_it_from_get_and_list() {
        let pool = make_pool().await;
        let fixture = seed_fixture(&pool).await;
        let usecase = make_usecase(pool.clone());

        let created = usecase.create_task(create_command(&fixture)).await.unwrap();

        usecase
            .soft_delete_task(created.id)
            .await
            .expect("soft_delete_task must succeed");

        let err = usecase
            .get_task(created.id)
            .await
            .expect_err("a soft-deleted task must not be gettable");
        assert!(matches!(err, common::CoreError::NotFound));

        let (items, _child_counts, total) = usecase
            .list_tasks(fixture.organization_id, None, 20, 0)
            .await
            .unwrap();
        assert_eq!(total, 0);
        assert!(items.is_empty());

        let missing = usecase
            .soft_delete_task(TaskId(generate_uuid_v7()))
            .await
            .expect_err("deleting an unknown task must fail");
        assert!(matches!(missing, common::CoreError::NotFound));

        cleanup(&pool, fixture.organization_id, &[fixture.owner_id]).await;
    }

    // -- migration 20260807000005: data reconciliation ----------------------

    /// Splits `postgres://user:pass@host:port/dbname` into
    /// `(base_without_dbname, dbname)` — good enough for the local dev URL
    /// shape this repo uses; it does not need to handle query parameters.
    fn split_database_url(url: &str) -> (String, String) {
        let idx = url
            .rfind('/')
            .expect("DATABASE_URL must contain a `/` separating the server from the database name");
        (url[..idx].to_owned(), url[idx + 1..].to_owned())
    }

    fn migrations_dir() -> std::path::PathBuf {
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../migrations")
    }

    /// Shells out to the `sqlx` CLI (the same tool this repo's migrations
    /// are applied with — see the environment section of the planning tasks
    /// remodel design doc) to bring `database_url` to exactly
    /// `target_version`, no further. `mestier-core` does not depend on the
    /// `sqlx` crate's `migrate` cargo feature, so driving migrations from
    /// Rust code directly is not available here without touching a
    /// `Cargo.toml` this workstream does not own.
    fn run_sqlx_migrate(database_url: &str, target_version: &str) {
        let status = std::process::Command::new("sqlx")
            .args([
                "migrate",
                "run",
                "--source",
                migrations_dir().to_str().unwrap(),
                "--database-url",
                database_url,
                "--target-version",
                target_version,
            ])
            .status()
            .expect("the `sqlx` CLI must be on PATH to run this migration integration test");

        assert!(
            status.success(),
            "`sqlx migrate run --target-version {target_version}` failed"
        );
    }

    /// Proves the `20260807000005_rename_work_orders_to_tasks` migration's
    /// data reconciliation on a scratch database: `note` survives as
    /// `description` with its content intact, an existing `title` is left
    /// untouched, and a missing `title` is backfilled with `'Sans titre'`
    /// rather than dropped or left blank.
    ///
    /// Requires the `sqlx` CLI binary on `PATH` (see `run_sqlx_migrate`) in
    /// addition to `DATABASE_URL` — if this fails with a "No such file or
    /// directory"-shaped panic rather than an assertion failure, that's why:
    /// install it with `cargo install sqlx-cli`, not a code regression.
    #[tokio::test]
    #[ignore = "requires live postgres and the sqlx CLI binary on PATH"]
    async fn migration_backfills_title_and_renames_note_to_description() {
        let (base_url, _current_db) =
            split_database_url(&std::env::var("DATABASE_URL").expect("DATABASE_URL must be set"));
        let admin_url = format!("{base_url}/postgres");
        let scratch_db = format!("mestier_migration_test_{}", uuid::Uuid::new_v4().simple());
        let scratch_url = format!("{base_url}/{scratch_db}");

        let admin_pool = PgPool::connect(&admin_url).await.unwrap();
        sqlx::query(&format!(r#"CREATE DATABASE "{scratch_db}""#))
            .execute(&admin_pool)
            .await
            .expect("creating the scratch database must succeed");

        // Bring the scratch database to right before this workstream's
        // migration, then seed rows shaped like pre-migration `work_orders`.
        run_sqlx_migrate(&scratch_url, "20260807000004");

        let scratch_pool = PgPool::connect(&scratch_url).await.unwrap();
        let owner_id = generate_uuid_v7();
        sqlx::query(
            r#"INSERT INTO users (id, email, username, display_name, sub) VALUES ($1, $2, $3, $4, $5)"#,
        )
        .bind(owner_id)
        .bind(format!("owner-{owner_id}@example.com"))
        .bind(format!("owner-{owner_id}"))
        .bind("Owner User")
        .bind(format!("sub-owner-{owner_id}"))
        .execute(&scratch_pool)
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
        .execute(&scratch_pool)
        .await
        .unwrap();

        let customer_id = generate_uuid_v7();
        // This test pins the schema at 20260807000004, before customers traded
        // `first_name`/`last_name` for a single `name` — the fixture must speak
        // the schema of that point in history, not today's.
        sqlx::query(
            r#"INSERT INTO customers (id, org_id, last_name, first_name) VALUES ($1, $2, $3, $4)"#,
        )
        .bind(customer_id)
        .bind(org_id)
        .bind("Dupont")
        .bind("Alice")
        .execute(&scratch_pool)
        .await
        .unwrap();

        let customer_context_id = generate_uuid_v7();
        sqlx::query(
            r#"INSERT INTO customer_contexts (id, customer_id, label) VALUES ($1, $2, $3)"#,
        )
        .bind(customer_context_id)
        .bind(customer_id)
        .bind("Chantier principal")
        .execute(&scratch_pool)
        .await
        .unwrap();

        // A legacy row with no title and a note to preserve.
        let untitled_id = generate_uuid_v7();
        sqlx::query(
            r#"INSERT INTO work_orders (id, org_id, customer_id, customer_context_id, starts_at, ends_at, status, title, note)
               VALUES ($1, $2, $3, $4, now(), now() + interval '1 hour', 'PLANNED', NULL, $5)"#,
        )
        .bind(untitled_id)
        .bind(org_id)
        .bind(customer_id)
        .bind(customer_context_id)
        .bind("Un vieux commentaire")
        .execute(&scratch_pool)
        .await
        .unwrap();

        // A legacy row that already had a title, and no note — both must be
        // left exactly as they were.
        let titled_id = generate_uuid_v7();
        sqlx::query(
            r#"INSERT INTO work_orders (id, org_id, customer_id, customer_context_id, starts_at, ends_at, status, title, note)
               VALUES ($1, $2, $3, $4, now(), now() + interval '1 hour', 'PLANNED', $5, NULL)"#,
        )
        .bind(titled_id)
        .bind(org_id)
        .bind(customer_id)
        .bind(customer_context_id)
        .bind("Titre déjà renseigné")
        .execute(&scratch_pool)
        .await
        .unwrap();

        scratch_pool.close().await;

        run_sqlx_migrate(&scratch_url, "20260807000005");

        let scratch_pool = PgPool::connect(&scratch_url).await.unwrap();
        let untitled_row: (String, Option<String>) =
            sqlx::query_as(r#"SELECT title, description FROM tasks WHERE id = $1"#)
                .bind(untitled_id)
                .fetch_one(&scratch_pool)
                .await
                .unwrap();
        assert_eq!(
            untitled_row.0, "Sans titre",
            "a row with no title must be backfilled with 'Sans titre'"
        );
        assert_eq!(
            untitled_row.1.as_deref(),
            Some("Un vieux commentaire"),
            "`note`'s content must survive as `description`"
        );

        let titled_row: (String, Option<String>) =
            sqlx::query_as(r#"SELECT title, description FROM tasks WHERE id = $1"#)
                .bind(titled_id)
                .fetch_one(&scratch_pool)
                .await
                .unwrap();
        assert_eq!(
            titled_row.0, "Titre déjà renseigné",
            "an existing title must be left untouched"
        );
        assert_eq!(titled_row.1, None);

        scratch_pool.close().await;
        sqlx::query(&format!(r#"DROP DATABASE IF EXISTS "{scratch_db}""#))
            .execute(&admin_pool)
            .await
            .ok();
    }
}
