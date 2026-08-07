#[cfg(test)]
#[allow(clippy::module_inception)]
mod tests {
    use std::sync::Arc;

    use chrono::{Duration, Utc};
    use common::{OrganizationId, UserId, generate_uuid_v7};
    use sqlx::PgPool;

    use crate::application::{MestierUseCase, default_authorizer};
    use crate::domain::work_order::{
        AssigneeRef,
        commands::{CreateWorkOrderCommand, PatchWorkOrderCommand},
    };
    use crate::infrastructure::realtime::{EventHub, RealtimeEventPublisher};
    use crate::{CustomerContextId, CustomerId, EmployeeId, WorkOrderId, WorkOrderStatus};

    async fn make_pool() -> PgPool {
        let url = std::env::var("DATABASE_URL")
            .expect("DATABASE_URL must be set to run work_order integration tests");
        PgPool::connect(&url).await.unwrap()
    }

    fn make_usecase(pool: PgPool) -> MestierUseCase {
        let hub = EventHub::new();
        let publisher = Arc::new(RealtimeEventPublisher::new(hub));
        MestierUseCase::new(pool, default_authorizer(), publisher)
    }

    struct Fixture {
        organization_id: OrganizationId,
        customer_id: CustomerId,
        customer_context_id: CustomerContextId,
        owner_id: UserId,
    }

    /// Seeds an owner user, an organization, a customer and a customer
    /// context — the minimal graph a work order needs to exist.
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
            r#"INSERT INTO customers (id, org_id, last_name, first_name)
               VALUES ($1, $2, $3, $4)"#,
            customer_id,
            org_id,
            "Dupont",
            "Alice",
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
            r#"INSERT INTO employees (id, org_id, name, hourly_rate_cents, weekly_contract_minutes)
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
            r#"INSERT INTO organization_members (organization_id, user_id)
               VALUES ($1, $2)"#,
            organization_id.0,
            user_id,
        )
        .execute(pool)
        .await
        .unwrap();

        UserId(user_id)
    }

    /// Removes everything seeded under `organization_id`, cascading to
    /// customers/customer_contexts/employees/work_orders/assignments/members,
    /// plus the loose user rows that outlive the organization.
    async fn cleanup(pool: &PgPool, organization_id: OrganizationId, user_ids: &[UserId]) {
        sqlx::query!(
            "DELETE FROM assignments WHERE org_id = $1",
            organization_id.0
        )
        .execute(pool)
        .await
        .ok();
        sqlx::query!(
            "DELETE FROM work_orders WHERE org_id = $1",
            organization_id.0
        )
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

    fn create_command(fixture: &Fixture) -> CreateWorkOrderCommand {
        let now = Utc::now();
        CreateWorkOrderCommand {
            organization_id: fixture.organization_id,
            customer_id: fixture.customer_id,
            customer_context_id: fixture.customer_context_id,
            quote_id: None,
            starts_at: now,
            ends_at: now + Duration::hours(2),
            all_day: false,
            title: Some("Réfection toiture".to_owned()),
            note: None,
        }
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn create_work_order_persists_and_is_retrievable() {
        let pool = make_pool().await;
        let fixture = seed_fixture(&pool).await;
        let usecase = make_usecase(pool.clone());

        let created = usecase
            .create_work_order(create_command(&fixture))
            .await
            .expect("create_work_order must succeed");

        assert_eq!(created.status, WorkOrderStatus::Planned);
        assert!(created.assignments.is_empty());

        let fetched = usecase
            .get_work_order(created.id)
            .await
            .expect("get_work_order must succeed");
        assert_eq!(fetched.id, created.id);
        assert_eq!(fetched.title.as_deref(), Some("Réfection toiture"));

        cleanup(&pool, fixture.organization_id, &[fixture.owner_id]).await;
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn list_work_orders_returns_only_the_organizations_own_orders() {
        let pool = make_pool().await;
        let fixture_a = seed_fixture(&pool).await;
        let fixture_b = seed_fixture(&pool).await;
        let usecase = make_usecase(pool.clone());

        usecase
            .create_work_order(create_command(&fixture_a))
            .await
            .unwrap();
        usecase
            .create_work_order(create_command(&fixture_b))
            .await
            .unwrap();

        let (items, total) = usecase
            .list_work_orders(fixture_a.organization_id, 20, 0)
            .await
            .expect("list_work_orders must succeed");

        assert_eq!(total, 1);
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].organization_id, fixture_a.organization_id);

        cleanup(&pool, fixture_a.organization_id, &[fixture_a.owner_id]).await;
        cleanup(&pool, fixture_b.organization_id, &[fixture_b.owner_id]).await;
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn patch_work_order_reschedules_reassigns_and_provisions_a_member_employee() {
        let pool = make_pool().await;
        let fixture = seed_fixture(&pool).await;
        let existing_employee_id = seed_employee(&pool, fixture.organization_id).await;
        let member_user_id = seed_member_without_employee(&pool, fixture.organization_id).await;
        let usecase = make_usecase(pool.clone());

        let created = usecase
            .create_work_order(create_command(&fixture))
            .await
            .unwrap();

        let new_starts_at = created.starts_at + Duration::days(1);
        let new_ends_at = created.ends_at + Duration::days(1);

        let mut patch = PatchWorkOrderCommand::new(created.id);
        patch.starts_at = Some(new_starts_at);
        patch.ends_at = Some(new_ends_at);
        patch.status = Some(WorkOrderStatus::InProgress);
        patch.assignees = Some(vec![
            AssigneeRef::Employee(existing_employee_id),
            AssigneeRef::Member(member_user_id),
        ]);

        let (updated, created_employees) = usecase
            .patch_work_order(patch)
            .await
            .expect("patch_work_order must succeed");

        assert_eq!(updated.starts_at, new_starts_at);
        assert_eq!(updated.ends_at, new_ends_at);
        assert_eq!(updated.status, WorkOrderStatus::InProgress);
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
        let mut second_patch = PatchWorkOrderCommand::new(created.id);
        second_patch.assignees = Some(vec![AssigneeRef::Member(member_user_id)]);

        let (reassigned, created_on_second_patch) = usecase
            .patch_work_order(second_patch)
            .await
            .expect("second patch_work_order must succeed");

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
    async fn patch_work_order_rolls_back_the_whole_transaction_on_failure() {
        let pool = make_pool().await;
        let fixture = seed_fixture(&pool).await;
        let member_user_id = seed_member_without_employee(&pool, fixture.organization_id).await;
        let usecase = make_usecase(pool.clone());

        let created = usecase
            .create_work_order(create_command(&fixture))
            .await
            .unwrap();
        let original_starts_at = created.starts_at;
        let original_ends_at = created.ends_at;

        let bogus_employee_id = EmployeeId(generate_uuid_v7());
        let mut patch = PatchWorkOrderCommand::new(created.id);
        patch.starts_at = Some(original_starts_at + Duration::days(1));
        patch.ends_at = Some(original_ends_at + Duration::days(1));
        // First assignee provisions a real employee record (a real write
        // inside the transaction); the second references an employee that
        // does not exist, which fails the whole `PATCH` after that write
        // already happened. Nothing must survive the rollback.
        patch.assignees = Some(vec![
            AssigneeRef::Member(member_user_id),
            AssigneeRef::Employee(bogus_employee_id),
        ]);

        let err = usecase
            .patch_work_order(patch)
            .await
            .expect_err("patch_work_order must fail for an unknown employee assignee");
        assert!(matches!(err, common::CoreError::NotFound));

        let work_order = usecase.get_work_order(created.id).await.unwrap();
        assert_eq!(
            work_order.starts_at, original_starts_at,
            "the reschedule must not have landed"
        );
        assert_eq!(work_order.ends_at, original_ends_at);
        assert!(
            work_order.assignments.is_empty(),
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
    async fn soft_delete_work_order_hides_it_from_get_and_list() {
        let pool = make_pool().await;
        let fixture = seed_fixture(&pool).await;
        let usecase = make_usecase(pool.clone());

        let created = usecase
            .create_work_order(create_command(&fixture))
            .await
            .unwrap();

        usecase
            .soft_delete_work_order(created.id)
            .await
            .expect("soft_delete_work_order must succeed");

        let err = usecase
            .get_work_order(created.id)
            .await
            .expect_err("a soft-deleted work order must not be gettable");
        assert!(matches!(err, common::CoreError::NotFound));

        let (items, total) = usecase
            .list_work_orders(fixture.organization_id, 20, 0)
            .await
            .unwrap();
        assert_eq!(total, 0);
        assert!(items.is_empty());

        let missing = usecase
            .soft_delete_work_order(WorkOrderId(generate_uuid_v7()))
            .await
            .expect_err("deleting an unknown work order must fail");
        assert!(matches!(missing, common::CoreError::NotFound));

        cleanup(&pool, fixture.organization_id, &[fixture.owner_id]).await;
    }
}
