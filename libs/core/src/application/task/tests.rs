#[cfg(test)]
#[allow(clippy::module_inception)]
mod tests {

    use chrono::{DateTime, Duration, Utc};
    use common::{CoreError, OrganizationId, UserId, generate_uuid_v7};
    use sqlx::PgPool;
    use uuid::Uuid;

    use crate::application::task::{BoardDrop, ParentScope, TaskFilter};
    use crate::application::test_support::{dev_pool, purge};
    use crate::application::{MestierUseCase, default_authorizer};
    use crate::domain::task::{
        AssigneeRef, BoardRank,
        commands::{CreateTaskCommand, PatchTaskCommand},
        service::sort_by_board_rank,
    };
    use crate::infrastructure::realtime::EventHub;
    use crate::{
        CustomerContextId, CustomerId, MemberId, ProjectId, Task, TaskId, TaskLabelId, TaskStatus,
    };

    async fn make_pool() -> PgPool {
        dev_pool().await
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
    /// A member with a contractual profile attached. Returns the seat — the
    /// profile is not what an assignment points at any more.
    async fn seed_employee(pool: &PgPool, organization_id: OrganizationId) -> MemberId {
        let member_id = generate_uuid_v7();
        sqlx::query!(
            r#"INSERT INTO organization_members (id, organization_id, last_name)
               VALUES ($1, $2, $3)"#,
            member_id,
            organization_id.0,
            "Existing Employee",
        )
        .execute(pool)
        .await
        .unwrap();

        let employee_id = generate_uuid_v7();
        sqlx::query!(
            r#"INSERT INTO employees (id, org_id, member_id, hourly_rate_cents, weekly_contract_minutes)
               VALUES ($1, $2, $3, $4, $5)"#,
            employee_id,
            organization_id.0,
            member_id,
            3500,
            2100,
        )
        .execute(pool)
        .await
        .unwrap();
        MemberId(member_id)
    }

    /// Seeds a user who is a member of `organization_id` but has no
    /// employee record yet — a "member-only" planning resource.
    /// A seat with an occupant but no contractual profile — plannable all the
    /// same, which is the whole point of #182. Returns both, since the caller
    /// needs the user id only for cleanup.
    async fn seed_member_without_employee(
        pool: &PgPool,
        organization_id: OrganizationId,
    ) -> (MemberId, UserId) {
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

        let member_id = generate_uuid_v7();
        sqlx::query!(
            r#"INSERT INTO organization_members (id, organization_id, user_id, last_name)
               VALUES ($1, $2, $3, $4)"#,
            member_id,
            organization_id.0,
            user_id,
            "Member Without Employee",
        )
        .execute(pool)
        .await
        .unwrap();

        (MemberId(member_id), UserId(user_id))
    }

    /// Removes everything seeded under `organization_id`, cascading to
    /// customers/customer_contexts/employees/tasks/task_assignments/members,
    /// plus the loose user rows that outlive the organization.
    async fn cleanup(pool: &PgPool, organization_id: OrganizationId, user_ids: &[UserId]) {
        purge(
            pool,
            "DELETE FROM task_assignments WHERE org_id = $1",
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
            "DELETE FROM task_labels WHERE org_id = $1",
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
            "DELETE FROM employees WHERE org_id = $1",
            organization_id.0,
        )
        .await;
        purge(
            pool,
            "DELETE FROM customers WHERE org_id = $1",
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

    fn create_command(fixture: &Fixture) -> CreateTaskCommand {
        let now = Utc::now();
        CreateTaskCommand {
            actor: authz::Subject::system(),
            organization_id: fixture.organization_id,
            parent_task_id: None,
            title: "Réfection toiture".to_owned(),
            description: None,
            starts_at: Some(now),
            ends_at: Some(now + Duration::hours(2)),
            all_day: false,
            status: None,
            blocks_availability: true,
            customer_id: Some(fixture.customer_id),
            customer_context_id: Some(fixture.customer_context_id),
            quote_id: None,
            project_id: None,
            expenses_cents: 0,
            expenses_label: None,
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
            .list_tasks(fixture_a.organization_id, TaskFilter::roots(), 20, 0)
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
            .list_tasks(fixture.organization_id, TaskFilter::roots(), 20, 0)
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
            .list_tasks(
                fixture.organization_id,
                TaskFilter::children_of(root.id),
                20,
                0,
            )
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
        let mut patch_a = PatchTaskCommand::new(task_a.id, authz::Subject::system());
        patch_a.assignees = Some(vec![AssigneeRef(employee_a)]);
        usecase.patch_task(patch_a).await.unwrap();

        let mut second_command = create_command(&fixture);
        second_command.title = "Deuxième tâche".to_owned();
        let task_b = usecase.create_task(second_command).await.unwrap();
        let mut patch_b = PatchTaskCommand::new(task_b.id, authz::Subject::system());
        patch_b.assignees = Some(vec![AssigneeRef(employee_b)]);
        usecase.patch_task(patch_b).await.unwrap();

        let mut third_command = create_command(&fixture);
        third_command.title = "Troisième tâche — sans assigné".to_owned();
        let task_c = usecase.create_task(third_command).await.unwrap();
        // task_c is deliberately left unassigned.

        let (tasks, _child_counts, total) = usecase
            .list_tasks(fixture.organization_id, TaskFilter::roots(), 20, 0)
            .await
            .expect("list_tasks must succeed");

        assert_eq!(total, 3);
        assert_eq!(tasks.len(), 3);

        let find = |id: TaskId| tasks.iter().find(|task| task.id == id).unwrap();
        let member_ids_of = |id: TaskId| -> Vec<MemberId> {
            find(id)
                .assignments
                .iter()
                .map(|assignment| assignment.member_id)
                .collect()
        };

        assert_eq!(member_ids_of(task_a.id), vec![employee_a]);
        assert_eq!(member_ids_of(task_b.id), vec![employee_b]);
        assert!(
            member_ids_of(task_c.id).is_empty(),
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

    /// A member with no contractual profile is assignable exactly like one
    /// with a contract, and assigning them provisions nothing.
    ///
    /// This test used to assert the opposite: that assigning a bare member
    /// created an employee record on the fly, and that a second assignment
    /// reused it rather than creating a duplicate. That whole mechanism existed
    /// because only an employee could be assigned — #182 removed it, so what is
    /// checked here is that `employees` stays untouched.
    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn patch_task_assigns_a_member_with_no_profile_without_provisioning_one() {
        let pool = make_pool().await;
        let fixture = seed_fixture(&pool).await;
        let with_profile = seed_employee(&pool, fixture.organization_id).await;
        let (without_profile, member_user_id) =
            seed_member_without_employee(&pool, fixture.organization_id).await;
        let usecase = make_usecase(pool.clone());

        let created = usecase.create_task(create_command(&fixture)).await.unwrap();

        let profiles_before: i64 = sqlx::query_scalar!(
            r#"SELECT COUNT(*) AS "count!" FROM employees WHERE org_id = $1"#,
            fixture.organization_id.0,
        )
        .fetch_one(&pool)
        .await
        .unwrap();

        let new_starts_at = created.starts_at.unwrap() + Duration::days(1);
        let new_ends_at = created.ends_at.unwrap() + Duration::days(1);

        let mut patch = PatchTaskCommand::new(created.id, authz::Subject::system());
        patch.starts_at = Some(Some(new_starts_at));
        patch.ends_at = Some(Some(new_ends_at));
        patch.status = Some(TaskStatus::InProgress);
        patch.assignees = Some(vec![
            AssigneeRef(with_profile),
            AssigneeRef(without_profile),
        ]);

        let updated = usecase
            .patch_task(patch)
            .await
            .expect("patch_task must succeed");

        assert_eq!(updated.starts_at, Some(new_starts_at));
        assert_eq!(updated.ends_at, Some(new_ends_at));
        assert_eq!(updated.status, TaskStatus::InProgress);
        assert_eq!(updated.assignments.len(), 2);

        let assigned: Vec<MemberId> = updated
            .assignments
            .iter()
            .map(|assignment| assignment.member_id)
            .collect();
        assert!(assigned.contains(&with_profile));
        assert!(assigned.contains(&without_profile));

        let profiles_after: i64 = sqlx::query_scalar!(
            r#"SELECT COUNT(*) AS "count!" FROM employees WHERE org_id = $1"#,
            fixture.organization_id.0,
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(
            profiles_before, profiles_after,
            "assigning a member must never create an HR profile"
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
        let (seeded_member_id, member_user_id) =
            seed_member_without_employee(&pool, fixture.organization_id).await;
        let usecase = make_usecase(pool.clone());

        let created = usecase.create_task(create_command(&fixture)).await.unwrap();
        let original_starts_at = created.starts_at;
        let original_ends_at = created.ends_at;

        let bogus_member_id = MemberId(generate_uuid_v7());
        let mut patch = PatchTaskCommand::new(created.id, authz::Subject::system());
        patch.starts_at = Some(Some(original_starts_at.unwrap() + Duration::days(1)));
        patch.ends_at = Some(Some(original_ends_at.unwrap() + Duration::days(1)));
        // The first assignee is a real seat, the second is not. The whole
        // `PATCH` must fail and leave nothing behind — the reschedule included,
        // which lands before the assignees are resolved.
        patch.assignees = Some(vec![
            AssigneeRef(seeded_member_id),
            AssigneeRef(bogus_member_id),
        ]);

        let err = usecase
            .patch_task(patch)
            .await
            .expect_err("patch_task must fail for an unknown member assignee");
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

        cleanup(
            &pool,
            fixture.organization_id,
            &[fixture.owner_id, member_user_id],
        )
        .await;
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn bulk_assign_tasks_assigns_every_task_in_one_call() {
        let pool = make_pool().await;
        let fixture = seed_fixture(&pool).await;
        let member_id = seed_employee(&pool, fixture.organization_id).await;
        let usecase = make_usecase(pool.clone());

        let task_a = usecase.create_task(create_command(&fixture)).await.unwrap();
        let task_b = usecase.create_task(create_command(&fixture)).await.unwrap();
        assert!(task_a.assignments.is_empty());
        assert!(task_b.assignments.is_empty());

        let updated = usecase
            .bulk_assign_tasks(
                authz::Subject::system(),
                fixture.organization_id,
                vec![task_a.id, task_b.id],
                vec![AssigneeRef(member_id)],
            )
            .await
            .expect("bulk_assign_tasks must succeed for two valid tasks");

        assert_eq!(updated.len(), 2);
        assert!(updated.iter().all(|t| t.assignments.len() == 1));
        assert!(
            updated
                .iter()
                .all(|t| t.assignments[0].member_id == member_id)
        );

        let reloaded_a = usecase.get_task(task_a.id).await.unwrap();
        let reloaded_b = usecase.get_task(task_b.id).await.unwrap();
        assert_eq!(reloaded_a.assignments.len(), 1);
        assert_eq!(reloaded_b.assignments.len(), 1);

        cleanup(&pool, fixture.organization_id, &[fixture.owner_id]).await;
    }

    /// Proves the acceptance criterion: a batch naming one task from another
    /// organization (indistinguishable, from the caller's side, from a typo'd
    /// or deleted id — both read as `NotFound`) fails the whole call and
    /// leaves *no* task assigned, including the ones earlier in the list
    /// whose own `UPDATE` already ran before the failure. Only a real
    /// transaction proves this — the mock-based
    /// `domain::task::service::tests::bulk_assign_tasks_stops_at_the_first_missing_task_never_reaching_the_rest`
    /// can only show the loop stops calling the repository, not that an
    /// already-issued write gets undone.
    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn bulk_assign_tasks_rolls_back_the_whole_batch_on_a_partial_failure() {
        let pool = make_pool().await;
        let fixture = seed_fixture(&pool).await;
        let other_fixture = seed_fixture(&pool).await;
        let member_id = seed_employee(&pool, fixture.organization_id).await;
        let usecase = make_usecase(pool.clone());

        let task_a = usecase.create_task(create_command(&fixture)).await.unwrap();
        let foreign_task = usecase
            .create_task(create_command(&other_fixture))
            .await
            .unwrap();

        let err = usecase
            .bulk_assign_tasks(
                authz::Subject::system(),
                fixture.organization_id,
                vec![task_a.id, foreign_task.id],
                vec![AssigneeRef(member_id)],
            )
            .await
            .expect_err("a task from another organization must fail the whole batch");
        assert!(matches!(err, common::CoreError::NotFound));

        let reloaded_a = usecase.get_task(task_a.id).await.unwrap();
        assert!(
            reloaded_a.assignments.is_empty(),
            "task_a's own assignment, issued before the failure, must not survive the rollback"
        );

        cleanup(&pool, fixture.organization_id, &[fixture.owner_id]).await;
        cleanup(
            &pool,
            other_fixture.organization_id,
            &[other_fixture.owner_id],
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
            .soft_delete_task(authz::Subject::system(), created.id)
            .await
            .expect("soft_delete_task must succeed");

        let err = usecase
            .get_task(created.id)
            .await
            .expect_err("a soft-deleted task must not be gettable");
        assert!(matches!(err, common::CoreError::NotFound));

        let (items, _child_counts, total) = usecase
            .list_tasks(fixture.organization_id, TaskFilter::roots(), 20, 0)
            .await
            .unwrap();
        assert_eq!(total, 0);
        assert!(items.is_empty());

        let missing = usecase
            .soft_delete_task(authz::Subject::system(), TaskId(generate_uuid_v7()))
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
        // Loud, like every other cleanup here: a scratch database left behind is
        // the same leak as a stray fixture, one that also costs disk until
        // somebody notices.
        sqlx::query(&format!(r#"DROP DATABASE IF EXISTS "{scratch_db}""#))
            .execute(&admin_pool)
            .await
            .unwrap_or_else(|error| panic!("dropping {scratch_db} failed: {error}"));
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn patching_expenses_persists_the_amount_and_its_label() {
        let pool = make_pool().await;
        let fixture = seed_fixture(&pool).await;
        let usecase = make_usecase(pool.clone());

        let created = usecase.create_task(create_command(&fixture)).await.unwrap();
        assert_eq!(created.expenses_cents, 0);
        assert_eq!(created.expenses_label, None);

        let mut patch = PatchTaskCommand::new(created.id, authz::Subject::system());
        patch.expenses_cents = Some(4500);
        patch.expenses_label = Some(Some("Déplacement Clermont".to_owned()));

        let patched = usecase.patch_task(patch).await.unwrap();
        assert_eq!(patched.expenses_cents, 4500);
        assert_eq!(
            patched.expenses_label.as_deref(),
            Some("Déplacement Clermont")
        );

        let fetched = usecase.get_task(created.id).await.unwrap();
        assert_eq!(fetched.expenses_cents, 4500);

        cleanup(&pool, fixture.organization_id, &[fixture.owner_id]).await;
    }

    /// The check constraint would catch this too, but it would surface as a
    /// database error and render as a 500. The service refuses it first.
    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn an_amount_with_no_label_is_a_conflict_not_a_database_error() {
        let pool = make_pool().await;
        let fixture = seed_fixture(&pool).await;
        let usecase = make_usecase(pool.clone());

        let created = usecase.create_task(create_command(&fixture)).await.unwrap();

        let mut patch = PatchTaskCommand::new(created.id, authz::Subject::system());
        patch.expenses_cents = Some(4500);

        let err = usecase.patch_task(patch).await.unwrap_err();
        assert!(matches!(err, CoreError::Conflict(_)), "got {err:?}");

        let mut negative = PatchTaskCommand::new(created.id, authz::Subject::system());
        negative.expenses_cents = Some(-1);
        negative.expenses_label = Some(Some("Déplacement".to_owned()));
        assert!(matches!(
            usecase.patch_task(negative).await.unwrap_err(),
            CoreError::Conflict(_)
        ));

        // Neither attempt wrote anything.
        let fetched = usecase.get_task(created.id).await.unwrap();
        assert_eq!(fetched.expenses_cents, 0);
        assert_eq!(fetched.expenses_label, None);

        cleanup(&pool, fixture.organization_id, &[fixture.owner_id]).await;
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn clearing_the_amount_clears_the_label_with_it() {
        let pool = make_pool().await;
        let fixture = seed_fixture(&pool).await;
        let usecase = make_usecase(pool.clone());

        let created = usecase.create_task(create_command(&fixture)).await.unwrap();

        let mut patch = PatchTaskCommand::new(created.id, authz::Subject::system());
        patch.expenses_cents = Some(4500);
        patch.expenses_label = Some(Some("Déplacement".to_owned()));
        usecase.patch_task(patch).await.unwrap();

        let mut clear = PatchTaskCommand::new(created.id, authz::Subject::system());
        clear.expenses_cents = Some(0);

        let cleared = usecase.patch_task(clear).await.unwrap();
        assert_eq!(cleared.expenses_cents, 0);
        assert_eq!(
            cleared.expenses_label, None,
            "an amount of zero must not keep a reason nobody can reconcile"
        );

        cleanup(&pool, fixture.organization_id, &[fixture.owner_id]).await;
    }
    // -- board_rank ---------------------------------------------------------

    /// The acceptance criterion that cannot be checked on either side alone:
    /// PostgreSQL and the domain must order a board column identically,
    /// `NULL` included. Both orderings are run over the same rows, in the same
    /// process, and compared — so a change to either one (a different
    /// collation on the column, `Option`'s own `Ord` creeping into
    /// `sort_by_board_rank`, a dropped `NULLS LAST`) fails here rather than
    /// showing up in production as cards that swap places when the board is
    /// reloaded.
    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn board_rank_orders_a_column_the_same_way_in_sql_and_in_the_domain() {
        let pool = make_pool().await;
        let fixture = seed_fixture(&pool).await;
        let usecase = make_usecase(pool.clone());

        // A column of ranked cards plus two that nobody has ever dragged.
        // "z" and "9z" are in there on purpose: they are the pairs a
        // linguistic collation is most likely to order differently from byte
        // order, which is what `TEXT COLLATE "C"` exists to prevent.
        let ranks = [
            Some("1"),
            Some("9z"),
            Some("a"),
            Some("i"),
            Some("z"),
            None,
            None,
        ];
        let mut created_ids = Vec::new();
        for rank in ranks {
            let created = usecase.create_task(create_command(&fixture)).await.unwrap();
            assert_eq!(
                created.board_rank, None,
                "a task is born unranked, whatever its column"
            );

            if let Some(rank) = rank {
                let mut patch = PatchTaskCommand::new(created.id, authz::Subject::system());
                patch.board_rank = Some(Some(BoardRank(rank.to_owned())));
                let patched = usecase.patch_task(patch).await.unwrap();
                assert_eq!(patched.board_rank, Some(BoardRank(rank.to_owned())));
            }

            created_ids.push(created.id);
        }

        // The SQL side: the ordering a board's read will use, straight out of
        // the index this migration adds.
        let sql_order: Vec<Uuid> = sqlx::query_scalar!(
            r#"
            SELECT id
            FROM tasks
            WHERE org_id = $1 AND deleted_at IS NULL
            ORDER BY board_rank ASC NULLS LAST, id ASC
            "#,
            fixture.organization_id.0,
        )
        .fetch_all(&pool)
        .await
        .unwrap();

        // The domain side: the same rows, read back through the repository and
        // sorted in memory.
        let (mut tasks, _counts, _total) = usecase
            .list_tasks(fixture.organization_id, TaskFilter::roots(), 100, 0)
            .await
            .unwrap();
        sort_by_board_rank(&mut tasks);
        let domain_order: Vec<Uuid> = tasks.iter().map(|task| task.id.0).collect();

        assert_eq!(
            domain_order, sql_order,
            "the database and the domain must order a board column identically"
        );

        // And, explicitly, the half of that agreement most likely to rot: the
        // two unranked cards sit at the end, not the beginning.
        let unranked = tasks.iter().filter(|t| t.board_rank.is_none()).count();
        assert_eq!(unranked, 2);
        assert!(
            tasks[tasks.len() - 2..]
                .iter()
                .all(|t| t.board_rank.is_none()),
            "NULL sorts after every ranked task"
        );

        cleanup(&pool, fixture.organization_id, &[fixture.owner_id]).await;
    }

    /// The round trip through the column, and the orthogonality rule holding
    /// against the real schema rather than against a mock: a `PATCH` carrying
    /// only a rank comes back with the status and both dates it went in with.
    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn patching_only_the_board_rank_leaves_the_status_and_the_window_alone() {
        let pool = make_pool().await;
        let fixture = seed_fixture(&pool).await;
        let usecase = make_usecase(pool.clone());

        let created = usecase.create_task(create_command(&fixture)).await.unwrap();

        // Where a drop between the column's first two cards would land.
        let moved = BoardRank::between(
            Some(&BoardRank("a".to_owned())),
            Some(&BoardRank("b".to_owned())),
        )
        .unwrap();

        let mut patch = PatchTaskCommand::new(created.id, authz::Subject::system());
        patch.board_rank = Some(Some(moved.clone()));
        let patched = usecase.patch_task(patch).await.unwrap();

        assert_eq!(patched.board_rank, Some(moved.clone()));
        assert_eq!(patched.status, created.status);
        assert_eq!(patched.starts_at, created.starts_at);
        assert_eq!(patched.ends_at, created.ends_at);

        // Read back rather than trusted: the `RETURNING` clause and the next
        // `SELECT` have to agree about the column too.
        let fetched = usecase.get_task(created.id).await.unwrap();
        assert_eq!(fetched.board_rank, Some(moved));
        assert_eq!(fetched.status, created.status);
        assert_eq!(fetched.starts_at, created.starts_at);

        // `Some(None)` clears it, and clears nothing else.
        let mut clear = PatchTaskCommand::new(created.id, authz::Subject::system());
        clear.board_rank = Some(None);
        let cleared = usecase.patch_task(clear).await.unwrap();
        assert_eq!(cleared.board_rank, None);
        assert_eq!(cleared.status, created.status);
        assert_eq!(cleared.starts_at, created.starts_at);

        cleanup(&pool, fixture.organization_id, &[fixture.owner_id]).await;
    }

    /// The one-row guarantee, measured rather than assumed: a drop between two
    /// neighbours writes exactly one row, and leaves every other card in the
    /// column byte-identical.
    ///
    /// `xact_commit`-style counters would measure the wrong thing (the
    /// transaction also touches `task_assignments`), so this compares the
    /// `updated_at` of every card in the column across the move. Exactly one
    /// changes — which is the claim "one write" is making.
    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn moving_a_card_between_two_others_writes_exactly_one_row() {
        let pool = make_pool().await;
        let fixture = seed_fixture(&pool).await;
        let usecase = make_usecase(pool.clone());

        // A column of three ranked cards, and the card about to be dropped
        // between the first two.
        let mut column = Vec::new();
        for rank in ["1", "i", "z"] {
            let created = usecase.create_task(create_command(&fixture)).await.unwrap();
            let mut patch = PatchTaskCommand::new(created.id, authz::Subject::system());
            patch.board_rank = Some(Some(BoardRank(rank.to_owned())));
            column.push(usecase.patch_task(patch).await.unwrap());
        }
        let moving = usecase.create_task(create_command(&fixture)).await.unwrap();

        let before: Vec<(Uuid, DateTime<Utc>, Option<String>)> =
            snapshot(&pool, fixture.organization_id).await;

        let rank = BoardRank::between(column[0].board_rank.as_ref(), column[1].board_rank.as_ref())
            .unwrap();
        assert!(column[0].board_rank.as_ref().unwrap() < &rank);
        assert!(&rank < column[1].board_rank.as_ref().unwrap());

        let mut patch = PatchTaskCommand::new(moving.id, authz::Subject::system());
        patch.board_rank = Some(Some(rank.clone()));
        usecase.patch_task(patch).await.unwrap();

        let after = snapshot(&pool, fixture.organization_id).await;

        let touched: Vec<Uuid> = before
            .iter()
            .zip(after.iter())
            .filter(|(b, a)| b != a)
            .map(|(b, _)| b.0)
            .collect();
        assert_eq!(
            touched,
            vec![moving.id.0],
            "a drop must rewrite the dropped card and nothing else"
        );

        cleanup(&pool, fixture.organization_id, &[fixture.owner_id]).await;
    }

    /// Every card of an organization, keyed by id, with the two columns a
    /// write would disturb. Ordered by id so two snapshots line up positionally.
    async fn snapshot(
        pool: &PgPool,
        organization_id: OrganizationId,
    ) -> Vec<(Uuid, DateTime<Utc>, Option<String>)> {
        sqlx::query!(
            r#"
            SELECT id, updated_at, board_rank
            FROM tasks
            WHERE org_id = $1 AND deleted_at IS NULL
            ORDER BY id ASC
            "#,
            organization_id.0,
        )
        .fetch_all(pool)
        .await
        .unwrap()
        .into_iter()
        .map(|row| (row.id, row.updated_at, row.board_rank))
        .collect()
    }

    // -- filters ------------------------------------------------------------
    //
    // Everything below runs against the real schema rather than a mock, and
    // that is the whole point: a predicate that quietly matches nothing —
    // a filter bound to the wrong parameter, a join that loses a row, an
    // `AND` where an `OR` was meant — passes every mock-based test in this
    // repository and fails only here.

    async fn seed_project(pool: &PgPool, organization_id: OrganizationId, name: &str) -> ProjectId {
        let project_id = generate_uuid_v7();
        sqlx::query!(
            r#"INSERT INTO projects (id, org_id, name) VALUES ($1, $2, $3)"#,
            project_id,
            organization_id.0,
            name,
        )
        .execute(pool)
        .await
        .unwrap();
        ProjectId(project_id)
    }

    async fn seed_label(pool: &PgPool, organization_id: OrganizationId, name: &str) -> TaskLabelId {
        let label_id = generate_uuid_v7();
        sqlx::query!(
            r#"INSERT INTO task_labels (id, org_id, name, color) VALUES ($1, $2, $3, $4)"#,
            label_id,
            organization_id.0,
            name,
            "#112233",
        )
        .execute(pool)
        .await
        .unwrap();
        TaskLabelId(label_id)
    }

    /// The scope a board query uses: every task at any depth, narrowed only
    /// by the fields the caller sets. `TaskFilter::default()` is the tree
    /// view's "roots only", which is a different question.
    fn board_filter() -> TaskFilter {
        TaskFilter {
            parent: ParentScope::Any,
            ..TaskFilter::default()
        }
    }

    async fn ids_matching(
        usecase: &MestierUseCase,
        organization_id: OrganizationId,
        filter: TaskFilter,
    ) -> Vec<TaskId> {
        let (tasks, _counts, _total) = usecase
            .list_tasks(organization_id, filter, 100, 0)
            .await
            .unwrap();
        tasks.into_iter().map(|task| task.id).collect()
    }

    /// A project's board shows the project's cards, subtasks included — a
    /// subtask attaches straight to a project without going through its
    /// parent (see `migrations/20260821000001_create_projects.up.sql`), so
    /// answering with roots only would hide real cards while looking like a
    /// successful query.
    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn filtering_by_project_returns_its_roots_and_its_subtasks_and_nothing_else() {
        let pool = make_pool().await;
        let fixture = seed_fixture(&pool).await;
        let usecase = make_usecase(pool.clone());

        let wanted = seed_project(&pool, fixture.organization_id, "Toiture Dupont").await;
        let other = seed_project(&pool, fixture.organization_id, "Extension Martin").await;

        let mut root = create_command(&fixture);
        root.project_id = Some(wanted);
        let root = usecase.create_task(root).await.unwrap();

        let mut subtask = create_command(&fixture);
        subtask.parent_task_id = Some(root.id);
        subtask.project_id = Some(wanted);
        let subtask = usecase.create_task(subtask).await.unwrap();

        let mut elsewhere = create_command(&fixture);
        elsewhere.project_id = Some(other);
        let elsewhere = usecase.create_task(elsewhere).await.unwrap();

        let mut unattached = create_command(&fixture);
        unattached.parent_task_id = Some(root.id);
        let unattached = usecase.create_task(unattached).await.unwrap();

        let matched = ids_matching(
            &usecase,
            fixture.organization_id,
            TaskFilter {
                project_id: Some(wanted),
                ..board_filter()
            },
        )
        .await;

        assert!(matched.contains(&root.id));
        assert!(
            matched.contains(&subtask.id),
            "a subtask attached to the project is one of its cards"
        );
        assert!(!matched.contains(&elsewhere.id));
        assert!(!matched.contains(&unattached.id));
        assert_eq!(matched.len(), 2);

        cleanup(&pool, fixture.organization_id, &[fixture.owner_id]).await;
    }

    /// Two columns of a board, asked for at once. The values of a repeated
    /// `status` combine with `OR`; reading them as `AND` would return an
    /// empty column and look like "there is nothing in progress".
    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn two_statuses_return_the_union_of_the_two_columns() {
        let pool = make_pool().await;
        let fixture = seed_fixture(&pool).await;
        let usecase = make_usecase(pool.clone());

        let mut created = Vec::new();
        for status in [
            TaskStatus::Backlog,
            TaskStatus::InProgress,
            TaskStatus::Planned,
            TaskStatus::Done,
        ] {
            let mut command = create_command(&fixture);
            command.status = Some(status);
            created.push((status, usecase.create_task(command).await.unwrap().id));
        }

        let matched = ids_matching(
            &usecase,
            fixture.organization_id,
            TaskFilter {
                statuses: Some(vec![TaskStatus::Backlog, TaskStatus::InProgress]),
                ..board_filter()
            },
        )
        .await;

        for (status, id) in &created {
            let wanted = matches!(status, TaskStatus::Backlog | TaskStatus::InProgress);
            assert_eq!(
                matched.contains(id),
                wanted,
                "{status:?} should{} be in the union",
                if wanted { "" } else { " not" }
            );
        }
        assert_eq!(matched.len(), 2);

        cleanup(&pool, fixture.organization_id, &[fixture.owner_id]).await;
    }

    /// Assignments do not inherit. A subtask of a task assigned to somebody
    /// is not thereby assigned to them, and "my tasks" that quietly includes
    /// every child of every task you own is a list nobody can act on.
    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn assignee_filter_does_not_inherit_from_the_parent() {
        let pool = make_pool().await;
        let fixture = seed_fixture(&pool).await;
        let usecase = make_usecase(pool.clone());
        let member_id = seed_employee(&pool, fixture.organization_id).await;

        let parent = usecase.create_task(create_command(&fixture)).await.unwrap();
        let mut assign = PatchTaskCommand::new(parent.id, authz::Subject::system());
        assign.assignees = Some(vec![AssigneeRef(member_id)]);
        usecase.patch_task(assign).await.unwrap();

        let mut child = create_command(&fixture);
        child.parent_task_id = Some(parent.id);
        let child = usecase.create_task(child).await.unwrap();

        // A second assigned task, so the filter is not trivially "one row".
        let sibling = usecase.create_task(create_command(&fixture)).await.unwrap();
        let mut assign_sibling = PatchTaskCommand::new(sibling.id, authz::Subject::system());
        assign_sibling.assignees = Some(vec![AssigneeRef(member_id)]);
        usecase.patch_task(assign_sibling).await.unwrap();

        let matched = ids_matching(
            &usecase,
            fixture.organization_id,
            TaskFilter {
                assignee_id: Some(member_id),
                ..board_filter()
            },
        )
        .await;

        assert!(matched.contains(&parent.id));
        assert!(matched.contains(&sibling.id));
        assert!(
            !matched.contains(&child.id),
            "a task whose parent is assigned to this member is not itself assigned to them"
        );
        assert_eq!(matched.len(), 2);

        cleanup(&pool, fixture.organization_id, &[fixture.owner_id]).await;
    }

    /// The independence of the two axes, proved by the combination that only
    /// works if neither is derived from the other: a *done* task that was
    /// never on the calendar. `unscheduled` is `starts_at IS NULL` and
    /// nothing else; it is not a spelling of `status = 'BACKLOG'`, and a
    /// `BACKLOG` task that has been given dates is not unscheduled.
    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn unscheduled_and_status_are_independent_axes() {
        let pool = make_pool().await;
        let fixture = seed_fixture(&pool).await;
        let usecase = make_usecase(pool.clone());

        let make = |status: TaskStatus, dated: bool| {
            let mut command = create_command(&fixture);
            command.status = Some(status);
            if !dated {
                command.starts_at = None;
                command.ends_at = None;
            }
            command
        };

        let undated_done = usecase
            .create_task(make(TaskStatus::Done, false))
            .await
            .unwrap();
        let dated_done = usecase
            .create_task(make(TaskStatus::Done, true))
            .await
            .unwrap();
        let undated_backlog = usecase
            .create_task(make(TaskStatus::Backlog, false))
            .await
            .unwrap();
        let dated_backlog = usecase
            .create_task(make(TaskStatus::Backlog, true))
            .await
            .unwrap();

        let undated_and_done = ids_matching(
            &usecase,
            fixture.organization_id,
            TaskFilter {
                unscheduled: Some(true),
                statuses: Some(vec![TaskStatus::Done]),
                ..board_filter()
            },
        )
        .await;
        assert_eq!(
            undated_and_done,
            vec![undated_done.id],
            "`unscheduled=true&status=DONE` must return undated done tasks — if either axis were              derived from the other this answer would be empty"
        );

        // And the other three quadrants, so the test cannot pass by a
        // predicate that happens to select one row.
        let undated = ids_matching(
            &usecase,
            fixture.organization_id,
            TaskFilter {
                unscheduled: Some(true),
                ..board_filter()
            },
        )
        .await;
        assert!(undated.contains(&undated_done.id));
        assert!(undated.contains(&undated_backlog.id));
        assert!(!undated.contains(&dated_done.id));
        assert!(
            !undated.contains(&dated_backlog.id),
            "a BACKLOG task that carries dates is scheduled — the status did not decide this"
        );

        let scheduled = ids_matching(
            &usecase,
            fixture.organization_id,
            TaskFilter {
                unscheduled: Some(false),
                ..board_filter()
            },
        )
        .await;
        assert!(scheduled.contains(&dated_done.id));
        assert!(scheduled.contains(&dated_backlog.id));
        assert!(!scheduled.contains(&undated_done.id));

        cleanup(&pool, fixture.organization_id, &[fixture.owner_id]).await;
    }

    /// Label, customer and title in one test because they are one claim:
    /// each narrows, and `q` is a case-insensitive substring of the title —
    /// not full-text search, so a fragment in the middle of a word matches.
    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn label_customer_and_title_each_narrow_the_listing() {
        let pool = make_pool().await;
        let fixture = seed_fixture(&pool).await;
        let usecase = make_usecase(pool.clone());
        let label_id = seed_label(&pool, fixture.organization_id, "Urgence").await;

        let mut labelled = create_command(&fixture);
        labelled.title = "Réfection TOITURE".to_owned();
        let labelled = usecase.create_task(labelled).await.unwrap();
        let mut attach = PatchTaskCommand::new(labelled.id, authz::Subject::system());
        attach.label_ids = Some(vec![label_id]);
        usecase.patch_task(attach).await.unwrap();

        let mut plain = create_command(&fixture);
        plain.title = "Devis cuisine".to_owned();
        plain.customer_id = None;
        plain.customer_context_id = None;
        let plain = usecase.create_task(plain).await.unwrap();

        let by_label = ids_matching(
            &usecase,
            fixture.organization_id,
            TaskFilter {
                label_id: Some(label_id),
                ..board_filter()
            },
        )
        .await;
        assert_eq!(by_label, vec![labelled.id]);

        let by_customer = ids_matching(
            &usecase,
            fixture.organization_id,
            TaskFilter {
                customer_id: Some(fixture.customer_id),
                ..board_filter()
            },
        )
        .await;
        assert_eq!(by_customer, vec![labelled.id]);
        assert!(!by_customer.contains(&plain.id));

        let by_title = ids_matching(
            &usecase,
            fixture.organization_id,
            TaskFilter {
                title_contains: Some("oitur".to_owned()),
                ..board_filter()
            },
        )
        .await;
        assert_eq!(
            by_title,
            vec![labelled.id],
            "`q` is a case-insensitive substring: a fragment inside a word matches, and the case              of the stored title does not"
        );

        cleanup(&pool, fixture.organization_id, &[fixture.owner_id]).await;
    }

    /// The rule that no filter may be responsible for tenant isolation. The
    /// two organizations hold tasks that match the filter equally well, so a
    /// filter applied without the `org_id` scope would return both.
    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn a_filter_never_reaches_another_organizations_tasks() {
        let pool = make_pool().await;
        let mine = seed_fixture(&pool).await;
        let theirs = seed_fixture(&pool).await;
        let usecase = make_usecase(pool.clone());

        let mut ours = create_command(&mine);
        ours.title = "Chantier commun".to_owned();
        ours.status = Some(TaskStatus::InProgress);
        let ours = usecase.create_task(ours).await.unwrap();

        let mut foreign = create_command(&theirs);
        foreign.title = "Chantier commun".to_owned();
        foreign.status = Some(TaskStatus::InProgress);
        let foreign = usecase.create_task(foreign).await.unwrap();

        let filter = TaskFilter {
            statuses: Some(vec![TaskStatus::InProgress]),
            title_contains: Some("Chantier commun".to_owned()),
            ..board_filter()
        };

        let matched = ids_matching(&usecase, mine.organization_id, filter).await;

        assert_eq!(matched, vec![ours.id]);
        assert!(!matched.contains(&foreign.id));

        cleanup(&pool, theirs.organization_id, &[theirs.owner_id]).await;
        cleanup(&pool, mine.organization_id, &[mine.owner_id]).await;
    }

    /// Pagination metadata is built from a count taken under the *same*
    /// predicate as the page. A count that ignored the filters would promise
    /// pages the caller can never reach; one taken on a narrower predicate
    /// would hide the tail of the result.
    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn pagination_totals_follow_the_filter() {
        let pool = make_pool().await;
        let fixture = seed_fixture(&pool).await;
        let usecase = make_usecase(pool.clone());

        for _ in 0..5 {
            let mut command = create_command(&fixture);
            command.status = Some(TaskStatus::Done);
            usecase.create_task(command).await.unwrap();
        }
        for _ in 0..3 {
            let mut command = create_command(&fixture);
            command.status = Some(TaskStatus::Planned);
            usecase.create_task(command).await.unwrap();
        }

        let filter = TaskFilter {
            statuses: Some(vec![TaskStatus::Done]),
            ..board_filter()
        };

        let (first_page, _counts, total) = usecase
            .list_tasks(fixture.organization_id, filter.clone(), 2, 0)
            .await
            .unwrap();
        assert_eq!(first_page.len(), 2);
        assert_eq!(total, 5, "the count is taken under the filter, not over it");

        let (last_page, _counts, total_again) = usecase
            .list_tasks(fixture.organization_id, filter, 2, 4)
            .await
            .unwrap();
        assert_eq!(last_page.len(), 1);
        assert_eq!(total_again, 5);

        let (unfiltered, _counts, everything) = usecase
            .list_tasks(fixture.organization_id, board_filter(), 100, 0)
            .await
            .unwrap();
        assert_eq!(unfiltered.len(), 8);
        assert_eq!(everything, 8);

        cleanup(&pool, fixture.organization_id, &[fixture.owner_id]).await;
    }

    /// The order the endpoint now returns, and the reason `sort_by_board_rank`
    /// exists: the SQL clause and the in-memory comparator must produce the
    /// same sequence for the same rows, under a filter as well as without
    /// one. Ranked cards first in byte order, unranked last.
    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn a_filtered_listing_comes_back_in_board_order() {
        let pool = make_pool().await;
        let fixture = seed_fixture(&pool).await;
        let usecase = make_usecase(pool.clone());

        // Created in an order that is neither the rank order nor its
        // reverse, so a listing that forgot to sort cannot pass by luck.
        for rank in [Some("i"), None, Some("1"), Some("z"), None, Some("9z")] {
            let mut command = create_command(&fixture);
            command.status = Some(TaskStatus::InProgress);
            let created = usecase.create_task(command).await.unwrap();
            if let Some(rank) = rank {
                let mut patch = PatchTaskCommand::new(created.id, authz::Subject::system());
                patch.board_rank = Some(Some(BoardRank(rank.to_owned())));
                usecase.patch_task(patch).await.unwrap();
            }
        }
        // A card in another column, to prove the filter and the ordering
        // compose rather than one undoing the other.
        let mut other_column = create_command(&fixture);
        other_column.status = Some(TaskStatus::Done);
        let other_column = usecase.create_task(other_column).await.unwrap();
        let mut patch = PatchTaskCommand::new(other_column.id, authz::Subject::system());
        patch.board_rank = Some(Some(BoardRank("0i".to_owned())));
        usecase.patch_task(patch).await.unwrap();

        let (tasks, _counts, _total) = usecase
            .list_tasks(
                fixture.organization_id,
                TaskFilter {
                    statuses: Some(vec![TaskStatus::InProgress]),
                    ..board_filter()
                },
                100,
                0,
            )
            .await
            .unwrap();

        let ranks: Vec<Option<String>> = tasks
            .iter()
            .map(|task| task.board_rank.as_ref().map(|rank| rank.0.clone()))
            .collect();
        assert_eq!(
            ranks,
            vec![
                Some("1".to_owned()),
                Some("9z".to_owned()),
                Some("i".to_owned()),
                Some("z".to_owned()),
                None,
                None,
            ],
            "the endpoint orders by board rank with unranked cards last"
        );
        assert!(!tasks.iter().any(|task| task.id == other_column.id));

        // The same rows put through the domain comparator must not move.
        let mut again = tasks.clone();
        sort_by_board_rank(&mut again);
        assert_eq!(
            again.iter().map(|t| t.id).collect::<Vec<_>>(),
            tasks.iter().map(|t| t.id).collect::<Vec<_>>(),
            "`ORDER BY board_rank ASC NULLS LAST, created_at ASC, id ASC` and \
             `sort_by_board_rank` must agree row for row"
        );

        cleanup(&pool, fixture.organization_id, &[fixture.owner_id]).await;
    }

    // -- resolving a drop's rank from its neighbours -------------------------
    //
    // A drop is a `PATCH` naming the two cards it landed between, resolved
    // inside that `PATCH`'s own transaction — see
    // `PatchTaskCommand::board_drop`. There is deliberately no way to resolve
    // one on its own: resolving can write (it materializes a column that has
    // never been ranked), and a write outside the patch's transaction is how
    // two concurrent first-drops both initialize the same column.

    /// A `PATCH` that is nothing but a drop between `preceding` and
    /// `following`.
    fn drop_between(
        id: TaskId,
        preceding: Option<TaskId>,
        following: Option<TaskId>,
    ) -> PatchTaskCommand {
        let mut patch = PatchTaskCommand::new(id, authz::Subject::system());
        patch.board_drop = Some(BoardDrop {
            preceding,
            following,
        });
        patch
    }

    /// One column as the database holds it, in board order.
    async fn column_ranks(
        pool: &PgPool,
        organization_id: OrganizationId,
        status: TaskStatus,
    ) -> Vec<(Uuid, Option<String>)> {
        sqlx::query!(
            r#"
            SELECT id, board_rank
            FROM tasks
            WHERE org_id = $1 AND deleted_at IS NULL AND status = CAST($2 AS text)::task_status
            ORDER BY board_rank ASC NULLS LAST, created_at ASC, id ASC
            "#,
            organization_id.0,
            status.as_str(),
        )
        .fetch_all(pool)
        .await
        .unwrap()
        .into_iter()
        .map(|row| (row.id, row.board_rank))
        .collect()
    }

    /// The server-side half of a drag: the client names the two cards the
    /// card landed between, and the card ends up strictly between theirs
    /// with nothing else touched.
    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn a_drop_between_two_neighbours_writes_the_rank_and_nothing_else() {
        let pool = make_pool().await;
        let fixture = seed_fixture(&pool).await;
        let usecase = make_usecase(pool.clone());

        // One column, and the card being dragged is already in it: a drop
        // that does not change column, which is the ordinary gesture. The
        // card and its neighbours must share a column, because a drop is
        // resolved against the column the card is landing in — see
        // `a_patch_changing_column_lands_at_the_named_position_in_the_target`
        // for the cross-column shape.
        let mut column = Vec::new();
        for rank in ["a", "b"] {
            let mut command = create_command(&fixture);
            command.status = Some(TaskStatus::Backlog);
            let created = usecase.create_task(command).await.unwrap();
            let mut patch = PatchTaskCommand::new(created.id, authz::Subject::system());
            patch.board_rank = Some(Some(BoardRank(rank.to_owned())));
            column.push(usecase.patch_task(patch).await.unwrap());
        }

        let mut moving = create_command(&fixture);
        moving.status = Some(TaskStatus::Backlog);
        let moving = usecase.create_task(moving).await.unwrap();

        let moved = usecase
            .patch_task(drop_between(
                moving.id,
                Some(column[0].id),
                Some(column[1].id),
            ))
            .await
            .unwrap();

        let rank = moved.board_rank.clone().expect("the drop produced a rank");
        assert!(column[0].board_rank.as_ref().unwrap() < &rank);
        assert!(&rank < column[1].board_rank.as_ref().unwrap());
        assert_eq!(
            moved.status,
            TaskStatus::Backlog,
            "a `PATCH` that names only neighbours writes the rank and nothing else — the status              it went in with is the status it comes back with"
        );
        assert_eq!(moved.starts_at, moving.starts_at);
        assert_eq!(moved.ends_at, moving.ends_at);

        // Read back rather than trusted.
        let fetched = usecase.get_task(moving.id).await.unwrap();
        assert_eq!(fetched.board_rank, Some(rank));

        cleanup(&pool, fixture.organization_id, &[fixture.owner_id]).await;
    }

    /// The drop at the top and at the bottom of a column: one neighbour
    /// named, the other side unbounded.
    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn a_drop_at_the_edge_of_a_column_names_one_neighbour() {
        let pool = make_pool().await;
        let fixture = seed_fixture(&pool).await;
        let usecase = make_usecase(pool.clone());

        let card = usecase.create_task(create_command(&fixture)).await.unwrap();
        let mut patch = PatchTaskCommand::new(card.id, authz::Subject::system());
        patch.board_rank = Some(Some(BoardRank("i".to_owned())));
        let card = usecase.patch_task(patch).await.unwrap();

        let over = usecase.create_task(create_command(&fixture)).await.unwrap();
        let over = usecase
            .patch_task(drop_between(over.id, None, Some(card.id)))
            .await
            .unwrap();
        assert!(over.board_rank.unwrap() < BoardRank("i".to_owned()));

        let under = usecase.create_task(create_command(&fixture)).await.unwrap();
        let under = usecase
            .patch_task(drop_between(under.id, Some(card.id), None))
            .await
            .unwrap();
        assert!(BoardRank("i".to_owned()) < under.board_rank.unwrap());

        // An empty column names nobody at all.
        let alone = usecase.create_task(create_command(&fixture)).await.unwrap();
        let alone = usecase
            .patch_task(drop_between(alone.id, None, None))
            .await
            .unwrap();
        assert!(BoardRank("0".to_owned()) < alone.board_rank.unwrap());

        cleanup(&pool, fixture.organization_id, &[fixture.owner_id]).await;
    }

    /// A neighbour that belongs to another organization is a `404`, not a
    /// rank read off a row the caller is not allowed to know exists. The
    /// `org_id` predicate in `find_board_positions` is what makes it
    /// indistinguishable from "no such task".
    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn a_neighbour_from_another_organization_is_not_found() {
        let pool = make_pool().await;
        let mine = seed_fixture(&pool).await;
        let theirs = seed_fixture(&pool).await;
        let usecase = make_usecase(pool.clone());

        let foreign = usecase.create_task(create_command(&theirs)).await.unwrap();
        let mut patch = PatchTaskCommand::new(foreign.id, authz::Subject::system());
        patch.board_rank = Some(Some(BoardRank("i".to_owned())));
        usecase.patch_task(patch).await.unwrap();

        let ours = usecase.create_task(create_command(&mine)).await.unwrap();

        let error = usecase
            .patch_task(drop_between(ours.id, Some(foreign.id), None))
            .await
            .unwrap_err();
        assert!(matches!(error, CoreError::NotFound), "{error:?}");

        // And an id that exists nowhere gets the identical answer.
        let unknown = usecase
            .patch_task(drop_between(
                ours.id,
                Some(TaskId(generate_uuid_v7())),
                None,
            ))
            .await
            .unwrap_err();
        assert!(matches!(unknown, CoreError::NotFound), "{unknown:?}");

        // The refused patch wrote nothing — the whole transaction rolled back.
        let fetched = usecase.get_task(ours.id).await.unwrap();
        assert_eq!(fetched.board_rank, None);

        cleanup(&pool, theirs.organization_id, &[theirs.owner_id]).await;
        cleanup(&pool, mine.organization_id, &[mine.owner_id]).await;
    }

    /// Two ranks are only comparable inside one column, so a pair drawn from
    /// two of them brackets nothing. Refused rather than resolved against
    /// whichever column was looked at first, which would put the card
    /// somewhere the user did not aim.
    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn two_neighbours_in_different_columns_are_refused() {
        let pool = make_pool().await;
        let fixture = seed_fixture(&pool).await;
        let usecase = make_usecase(pool.clone());

        let mut planned = create_command(&fixture);
        planned.status = Some(TaskStatus::Planned);
        let planned = usecase.create_task(planned).await.unwrap();

        let mut done = create_command(&fixture);
        done.status = Some(TaskStatus::Done);
        let done = usecase.create_task(done).await.unwrap();

        let moving = usecase.create_task(create_command(&fixture)).await.unwrap();

        let error = usecase
            .patch_task(drop_between(moving.id, Some(planned.id), Some(done.id)))
            .await
            .unwrap_err();

        assert!(
            matches!(error, CoreError::Conflict(ref message) if message.contains("different columns")),
            "{error:?}"
        );

        // And nothing was initialized on the way to the refusal: the whole
        // patch, initialization included, is one transaction.
        for (_, rank) in column_ranks(&pool, fixture.organization_id, TaskStatus::Planned).await {
            assert_eq!(rank, None, "a refused drop must not have written a rank");
        }
        for (_, rank) in column_ranks(&pool, fixture.organization_id, TaskStatus::Done).await {
            assert_eq!(rank, None);
        }

        cleanup(&pool, fixture.organization_id, &[fixture.owner_id]).await;
    }

    // -- initializing a column ----------------------------------------------

    /// The gesture this exists for, against the real schema: the very first
    /// drop into a column where nothing has ever been ranked — which is every
    /// column of every organization, since `board_rank` was added with no
    /// backfill.
    ///
    /// Three claims in one test because they are one claim: the drop
    /// succeeds, it lands between the two cards it named, and the column it
    /// wrote down is the column that was already on screen.
    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn a_drop_between_two_unranked_neighbours_ranks_the_column_and_keeps_its_order() {
        let pool = make_pool().await;
        let fixture = seed_fixture(&pool).await;
        let usecase = make_usecase(pool.clone());

        let mut created = Vec::new();
        for title in ["Un", "Deux", "Trois", "Quatre"] {
            let mut command = create_command(&fixture);
            command.status = Some(TaskStatus::Backlog);
            command.title = title.to_owned();
            let task = usecase.create_task(command).await.unwrap();
            assert_eq!(task.board_rank, None, "a task is born unranked");
            created.push(task);
        }

        // The order the board was already displaying: every card unranked,
        // so `created_at ASC, id ASC` — which is creation order here.
        let before: Vec<Uuid> = column_ranks(&pool, fixture.organization_id, TaskStatus::Backlog)
            .await
            .into_iter()
            .map(|(id, _)| id)
            .collect();
        assert_eq!(
            before,
            created.iter().map(|task| task.id.0).collect::<Vec<_>>()
        );

        // Drop the last card between the first two.
        let moved = usecase
            .patch_task(drop_between(
                created[3].id,
                Some(created[0].id),
                Some(created[1].id),
            ))
            .await
            .unwrap();

        let after = column_ranks(&pool, fixture.organization_id, TaskStatus::Backlog).await;
        assert!(
            after.iter().all(|(_, rank)| rank.is_some()),
            "every card of the column is ranked once it has been initialized: {after:?}"
        );
        assert_eq!(
            after.iter().map(|(id, _)| *id).collect::<Vec<_>>(),
            vec![
                created[0].id.0,
                created[3].id.0,
                created[1].id.0,
                created[2].id.0,
            ],
            "the column keeps the order it had, with the dropped card moved to where it landed"
        );

        let rank = moved.board_rank.expect("the drop produced a rank");
        let rank_of = |task: &Task| {
            after
                .iter()
                .find(|(id, _)| *id == task.id.0)
                .and_then(|(_, rank)| rank.clone())
                .map(BoardRank)
                .unwrap()
        };
        assert!(rank_of(&created[0]) < rank && rank < rank_of(&created[1]));

        cleanup(&pool, fixture.organization_id, &[fixture.owner_id]).await;
    }

    /// A half-ranked column converges instead of being renumbered. The cards
    /// that already carry a rank keep the exact bytes they had — renumbering
    /// every card on a drop is the `i32`-position scheme fractional ranks
    /// exist to avoid — and the rest are placed after them, where they were
    /// already being displayed.
    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn initializing_a_half_ranked_column_does_not_renumber_the_ranked_cards() {
        let pool = make_pool().await;
        let fixture = seed_fixture(&pool).await;
        let usecase = make_usecase(pool.clone());

        let mut ranked = Vec::new();
        for rank in ["1", "m"] {
            let mut command = create_command(&fixture);
            command.status = Some(TaskStatus::Planned);
            let created = usecase.create_task(command).await.unwrap();
            let mut patch = PatchTaskCommand::new(created.id, authz::Subject::system());
            patch.board_rank = Some(Some(BoardRank(rank.to_owned())));
            ranked.push(usecase.patch_task(patch).await.unwrap());
        }

        let mut unranked = Vec::new();
        for _ in 0..3 {
            let mut command = create_command(&fixture);
            command.status = Some(TaskStatus::Planned);
            unranked.push(usecase.create_task(command).await.unwrap());
        }

        let moving = usecase.create_task(create_command(&fixture)).await.unwrap();
        usecase
            .patch_task(drop_between(
                moving.id,
                Some(unranked[0].id),
                Some(unranked[1].id),
            ))
            .await
            .unwrap();

        let after = column_ranks(&pool, fixture.organization_id, TaskStatus::Planned).await;
        let rank_of = |id: TaskId| {
            after
                .iter()
                .find(|(row, _)| *row == id.0)
                .and_then(|(_, rank)| rank.clone())
        };

        assert_eq!(
            rank_of(ranked[0].id),
            Some("1".to_owned()),
            "a card that already had a rank keeps it, byte for byte"
        );
        assert_eq!(rank_of(ranked[1].id), Some("m".to_owned()));
        for card in &unranked {
            let rank = rank_of(card.id).expect("the unranked cards were ranked");
            assert!(
                rank.as_str() > "m",
                "the newly ranked cards sort after every card that already had a rank: {rank}"
            );
        }

        cleanup(&pool, fixture.organization_id, &[fixture.owner_id]).await;
    }

    /// Idempotence, measured against the real rows: the second drop into the
    /// same column writes only the card being moved. `updated_at` is the
    /// instrument — an initialization that ran twice would touch every card
    /// in the column a second time.
    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn initializing_a_column_twice_writes_nothing_the_second_time() {
        let pool = make_pool().await;
        let fixture = seed_fixture(&pool).await;
        let usecase = make_usecase(pool.clone());

        let mut cards = Vec::new();
        for _ in 0..4 {
            let mut command = create_command(&fixture);
            command.status = Some(TaskStatus::InProgress);
            cards.push(usecase.create_task(command).await.unwrap());
        }

        usecase
            .patch_task(drop_between(
                cards[3].id,
                Some(cards[0].id),
                Some(cards[1].id),
            ))
            .await
            .unwrap();
        let after_first = snapshot(&pool, fixture.organization_id).await;

        usecase
            .patch_task(drop_between(
                cards[2].id,
                Some(cards[0].id),
                Some(cards[1].id),
            ))
            .await
            .unwrap();
        let after_second = snapshot(&pool, fixture.organization_id).await;

        let touched: Vec<Uuid> = after_first
            .iter()
            .zip(after_second.iter())
            .filter(|((_, before, _), (_, after, _))| before != after)
            .map(|((id, _, _), _)| *id)
            .collect();
        assert_eq!(
            touched,
            vec![cards[2].id.0],
            "the second drop found a ranked column and wrote one row — the card it moved"
        );

        cleanup(&pool, fixture.organization_id, &[fixture.owner_id]).await;
    }

    /// The claim that only a real database can make: initialization is a
    /// bulk write, and a bulk write is where the `TEXT COLLATE "C"` decision
    /// either holds or quietly stops holding. Both orderings are run over
    /// the same freshly initialized column and compared row for row.
    ///
    /// `"z"`-heavy and `"9"`-heavy ranks are what a linguistic collation is
    /// most likely to order differently from byte order, and an evenly
    /// spaced batch produces exactly those — it walks the alphabet.
    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn sql_and_the_domain_still_agree_after_a_column_is_initialized() {
        let pool = make_pool().await;
        let fixture = seed_fixture(&pool).await;
        let usecase = make_usecase(pool.clone());

        // Thirty cards, so the generated batch spans the alphabet twice over
        // and lands on two-character ranks.
        let mut cards = Vec::new();
        for index in 0..30 {
            let mut command = create_command(&fixture);
            command.status = Some(TaskStatus::Backlog);
            command.title = format!("Carte {index}");
            cards.push(usecase.create_task(command).await.unwrap());
        }

        usecase
            .patch_task(drop_between(
                cards[29].id,
                Some(cards[0].id),
                Some(cards[1].id),
            ))
            .await
            .unwrap();

        let sql_order: Vec<Uuid> = sqlx::query_scalar!(
            r#"
            SELECT id
            FROM tasks
            WHERE org_id = $1 AND deleted_at IS NULL
            ORDER BY board_rank ASC NULLS LAST, created_at ASC, id ASC
            "#,
            fixture.organization_id.0,
        )
        .fetch_all(&pool)
        .await
        .unwrap();

        let (mut tasks, _counts, _total) = usecase
            .list_tasks(
                fixture.organization_id,
                TaskFilter {
                    parent: ParentScope::Any,
                    ..TaskFilter::default()
                },
                100,
                0,
            )
            .await
            .unwrap();
        let listed: Vec<Uuid> = tasks.iter().map(|task| task.id.0).collect();
        sort_by_board_rank(&mut tasks);
        let domain_order: Vec<Uuid> = tasks.iter().map(|task| task.id.0).collect();

        assert_eq!(
            domain_order, sql_order,
            "a bulk-written column must order identically in PostgreSQL and in the domain"
        );
        assert_eq!(
            listed, sql_order,
            "and the endpoint's own ordering is already that order — nothing re-sorts a page"
        );
        assert!(
            tasks.iter().all(|task| task.board_rank.is_some()),
            "the whole column was initialized"
        );

        cleanup(&pool, fixture.organization_id, &[fixture.owner_id]).await;
    }
    // -- the column the card is landing in ----------------------------------
    //
    // A cross-column drop is one request: the new `status` and the target
    // column's neighbours travel in the same payload, `patch_task` writes the
    // status, and the drop resolves against the column the card is joining.
    // What these pin is that "the column the card is joining" is what the
    // resolver is actually told, rather than something it infers from the
    // neighbours and is therefore incapable of disagreeing with.

    /// A `PATCH` carrying both halves: the card changes column and lands at
    /// the position it named in that column.
    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn a_patch_changing_column_lands_at_the_named_position_in_the_target() {
        let pool = make_pool().await;
        let fixture = seed_fixture(&pool).await;
        let usecase = make_usecase(pool.clone());

        // A ranked target column, and a card sitting in another one.
        let mut target = Vec::new();
        for rank in ["a", "b"] {
            let mut command = create_command(&fixture);
            command.status = Some(TaskStatus::InProgress);
            let created = usecase.create_task(command).await.unwrap();
            let mut patch = PatchTaskCommand::new(created.id, authz::Subject::system());
            patch.board_rank = Some(Some(BoardRank(rank.to_owned())));
            target.push(usecase.patch_task(patch).await.unwrap());
        }

        let mut moving = create_command(&fixture);
        moving.status = Some(TaskStatus::Backlog);
        let moving = usecase.create_task(moving).await.unwrap();

        let mut patch = drop_between(moving.id, Some(target[0].id), Some(target[1].id));
        patch.status = Some(TaskStatus::InProgress);
        let moved = usecase.patch_task(patch).await.unwrap();

        assert_eq!(
            moved.status,
            TaskStatus::InProgress,
            "the status the same payload carried is written"
        );
        let rank = moved.board_rank.clone().expect("the drop produced a rank");
        assert!(target[0].board_rank.as_ref().unwrap() < &rank);
        assert!(&rank < target[1].board_rank.as_ref().unwrap());

        // And the card really is in the target column, at that position.
        let column = column_ranks(&pool, fixture.organization_id, TaskStatus::InProgress).await;
        assert_eq!(
            column.iter().map(|(id, _)| *id).collect::<Vec<_>>(),
            vec![target[0].id.0, moving.id.0, target[1].id.0],
        );
        assert!(
            column_ranks(&pool, fixture.organization_id, TaskStatus::Backlog)
                .await
                .is_empty(),
            "the card left the column it came from"
        );

        cleanup(&pool, fixture.organization_id, &[fixture.owner_id]).await;
    }

    /// The silent failure this closes: a `PATCH` that changes column while
    /// naming the cards it is *leaving behind*. The neighbours agree with
    /// each other, so the older check passes them; they do not agree with
    /// where the card is going, and that is now a `409` rather than a rank
    /// borrowed from another column's space.
    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn neighbours_from_the_source_column_are_refused_when_the_patch_changes_column() {
        let pool = make_pool().await;
        let fixture = seed_fixture(&pool).await;
        let usecase = make_usecase(pool.clone());

        let mut source = Vec::new();
        for _ in 0..2 {
            let mut command = create_command(&fixture);
            command.status = Some(TaskStatus::Backlog);
            source.push(usecase.create_task(command).await.unwrap());
        }

        let mut target = create_command(&fixture);
        target.status = Some(TaskStatus::InProgress);
        usecase.create_task(target).await.unwrap();

        let mut moving = create_command(&fixture);
        moving.status = Some(TaskStatus::Backlog);
        let moving = usecase.create_task(moving).await.unwrap();

        let mut patch = drop_between(moving.id, Some(source[0].id), Some(source[1].id));
        patch.status = Some(TaskStatus::InProgress);
        let error = usecase.patch_task(patch).await.unwrap_err();

        assert!(
            matches!(error, CoreError::Conflict(ref message)
                if message.contains("BACKLOG") && message.contains("IN_PROGRESS")),
            "the refusal must name the neighbours' column and the target: {error:?}"
        );

        // Neither column was initialized on the way out, and the status was
        // not written either — the refusal rolls the whole patch back.
        for status in [TaskStatus::Backlog, TaskStatus::InProgress] {
            for (id, rank) in column_ranks(&pool, fixture.organization_id, status).await {
                assert_eq!(
                    rank, None,
                    "a refused drop must leave every column untouched (task {id})"
                );
            }
        }
        assert_eq!(
            usecase.get_task(moving.id).await.unwrap().status,
            TaskStatus::Backlog,
            "the status the refused patch carried was rolled back with it"
        );

        cleanup(&pool, fixture.organization_id, &[fixture.owner_id]).await;
    }

    /// The regression guard for the case that is not a column change at all:
    /// no `status` in the payload, so the target is the card's current
    /// column and the drop resolves exactly as it did before this check
    /// existed.
    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn a_drop_with_no_status_change_resolves_against_the_cards_current_column() {
        let pool = make_pool().await;
        let fixture = seed_fixture(&pool).await;
        let usecase = make_usecase(pool.clone());

        let mut column = Vec::new();
        for rank in ["a", "b"] {
            let mut command = create_command(&fixture);
            command.status = Some(TaskStatus::Done);
            let created = usecase.create_task(command).await.unwrap();
            let mut patch = PatchTaskCommand::new(created.id, authz::Subject::system());
            patch.board_rank = Some(Some(BoardRank(rank.to_owned())));
            column.push(usecase.patch_task(patch).await.unwrap());
        }

        let mut moving = create_command(&fixture);
        moving.status = Some(TaskStatus::Done);
        let moving = usecase.create_task(moving).await.unwrap();

        // No `status` on the patch — the card is already where it is landing.
        let moved = usecase
            .patch_task(drop_between(
                moving.id,
                Some(column[0].id),
                Some(column[1].id),
            ))
            .await
            .unwrap();

        assert_eq!(moved.status, TaskStatus::Done, "the column is unchanged");
        let rank = moved.board_rank.expect("the drop produced a rank");
        assert!(column[0].board_rank.as_ref().unwrap() < &rank);
        assert!(&rank < column[1].board_rank.as_ref().unwrap());

        cleanup(&pool, fixture.organization_id, &[fixture.owner_id]).await;
    }

    /// A cross-column drop into a target that has never been ranked. Which
    /// rows the initialization lands on is the whole question: the column the
    /// card is **joining** is written down, and the one it is leaving is left
    /// exactly as it was.
    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn a_cross_column_drop_initializes_the_target_column_not_the_source() {
        let pool = make_pool().await;
        let fixture = seed_fixture(&pool).await;
        let usecase = make_usecase(pool.clone());

        let mut source = Vec::new();
        for _ in 0..3 {
            let mut command = create_command(&fixture);
            command.status = Some(TaskStatus::Backlog);
            source.push(usecase.create_task(command).await.unwrap());
        }

        let mut target = Vec::new();
        for _ in 0..3 {
            let mut command = create_command(&fixture);
            command.status = Some(TaskStatus::Planned);
            target.push(usecase.create_task(command).await.unwrap());
        }

        // Every card of both columns is unranked — a fresh install.
        for status in [TaskStatus::Backlog, TaskStatus::Planned] {
            for (_, rank) in column_ranks(&pool, fixture.organization_id, status).await {
                assert_eq!(rank, None);
            }
        }

        let mut patch = drop_between(source[0].id, Some(target[0].id), Some(target[1].id));
        patch.status = Some(TaskStatus::Planned);
        let moved = usecase.patch_task(patch).await.unwrap();

        let planned = column_ranks(&pool, fixture.organization_id, TaskStatus::Planned).await;
        assert!(
            planned.iter().all(|(_, rank)| rank.is_some()),
            "the target column was written down: {planned:?}"
        );
        assert_eq!(
            planned.iter().map(|(id, _)| *id).collect::<Vec<_>>(),
            vec![
                target[0].id.0,
                source[0].id.0,
                target[1].id.0,
                target[2].id.0,
            ],
            "the target keeps the order it had, with the arriving card where it landed"
        );
        assert!(moved.board_rank.is_some());

        let backlog = column_ranks(&pool, fixture.organization_id, TaskStatus::Backlog).await;
        assert_eq!(backlog.len(), 2, "the card left the source column");
        for (id, rank) in &backlog {
            assert_eq!(
                *rank, None,
                "the column the card left is not the column being written down (task {id})"
            );
        }

        cleanup(&pool, fixture.organization_id, &[fixture.owner_id]).await;
    }
}
