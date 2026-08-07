#[cfg(test)]
#[allow(clippy::module_inception)]
mod tests {
    use std::sync::Arc;

    use chrono::{DateTime, Duration, Utc};
    use common::{OrganizationId, UserId, generate_uuid_v7};
    use sqlx::PgPool;

    use crate::application::{MestierUseCase, default_authorizer};
    use crate::domain::planning::TimeRange;
    use crate::infrastructure::realtime::{EventHub, RealtimeEventPublisher};
    use crate::{CustomerContextId, CustomerId, DateRange, EmployeeId, PlanningResource};

    async fn make_pool() -> PgPool {
        let url = std::env::var("DATABASE_URL")
            .expect("DATABASE_URL must be set to run planning integration tests");
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

    /// Seeds a user and an active employee record linked to it — a person
    /// who is both a member and an employee, once `seed_member` also adds
    /// them to `organization_members`.
    async fn seed_employee_with_user(pool: &PgPool, organization_id: OrganizationId) -> UserId {
        let user_id = generate_uuid_v7();
        sqlx::query!(
            r#"INSERT INTO users (id, email, username, display_name, sub)
               VALUES ($1, $2, $3, $4, $5)"#,
            user_id,
            format!("linked-{user_id}@example.com"),
            format!("linked-{user_id}"),
            "Linked Employee",
            format!("sub-linked-{user_id}"),
        )
        .execute(pool)
        .await
        .unwrap();

        sqlx::query!(
            r#"INSERT INTO employees (id, org_id, user_id, name, hourly_rate_cents, weekly_contract_minutes)
               VALUES ($1, $2, $3, $4, $5, $6)"#,
            generate_uuid_v7(),
            organization_id.0,
            user_id,
            "Linked Employee",
            3500,
            2100,
        )
        .execute(pool)
        .await
        .unwrap();

        UserId(user_id)
    }

    /// Seeds a plain active employee with no linked user account.
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

    /// Adds `user_id` as a member of `organization_id`.
    async fn seed_member(pool: &PgPool, organization_id: OrganizationId, user_id: UserId) {
        sqlx::query!(
            r#"INSERT INTO organization_members (organization_id, user_id)
               VALUES ($1, $2)"#,
            organization_id.0,
            user_id.0,
        )
        .execute(pool)
        .await
        .unwrap();
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

        seed_member(pool, organization_id, UserId(user_id)).await;

        UserId(user_id)
    }

    async fn seed_work_order(
        pool: &PgPool,
        fixture: &Fixture,
        employee_id: EmployeeId,
        starts_at: DateTime<Utc>,
        ends_at: DateTime<Utc>,
    ) {
        let work_order_id = generate_uuid_v7();
        sqlx::query!(
            r#"
            INSERT INTO work_orders (id, org_id, customer_id, customer_context_id, starts_at, ends_at, status, title)
            VALUES ($1, $2, $3, $4, $5, $6, 'PLANNED', $7)
            "#,
            work_order_id,
            fixture.organization_id.0,
            fixture.customer_id.0,
            fixture.customer_context_id.0,
            starts_at,
            ends_at,
            "Réfection toiture",
        )
        .execute(pool)
        .await
        .unwrap();

        sqlx::query!(
            r#"INSERT INTO assignments (id, org_id, work_order_id, employee_id)
               VALUES ($1, $2, $3, $4)"#,
            generate_uuid_v7(),
            fixture.organization_id.0,
            work_order_id,
            employee_id.0,
        )
        .execute(pool)
        .await
        .unwrap();
    }

    async fn seed_absence(
        pool: &PgPool,
        organization_id: OrganizationId,
        employee_id: EmployeeId,
        starts_at: DateTime<Utc>,
        ends_at: DateTime<Utc>,
    ) {
        sqlx::query!(
            r#"
            INSERT INTO employee_absences (id, org_id, employee_id, kind, starts_at, ends_at)
            VALUES ($1, $2, $3, 'LEAVE', $4, $5)
            "#,
            generate_uuid_v7(),
            organization_id.0,
            employee_id.0,
            starts_at,
            ends_at,
        )
        .execute(pool)
        .await
        .unwrap();
    }

    /// Removes everything seeded under `organization_id`, plus the loose
    /// user rows that outlive the organization.
    async fn cleanup(pool: &PgPool, organization_id: OrganizationId, user_ids: &[UserId]) {
        for table in [
            "employee_absences",
            "employee_work_slots",
            "assignments",
            "work_orders",
        ] {
            sqlx::query(&format!("DELETE FROM {table} WHERE org_id = $1"))
                .bind(organization_id.0)
                .execute(pool)
                .await
                .ok();
        }
        sqlx::query!(
            r#"DELETE FROM employee_rhythms WHERE org_id = $1"#,
            organization_id.0
        )
        .execute(pool)
        .await
        .ok();
        sqlx::query!(
            "DELETE FROM organization_members WHERE organization_id = $1",
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

    fn date_range(from: (i32, u32, u32), to: (i32, u32, u32)) -> DateRange {
        DateRange::new(
            chrono::NaiveDate::from_ymd_opt(from.0, from.1, from.2).unwrap(),
            chrono::NaiveDate::from_ymd_opt(to.0, to.1, to.2).unwrap(),
        )
        .unwrap()
    }

    /// The dedup rule's central promise: a person who is both a member and
    /// an employee must appear exactly once in the read model, on the
    /// employee side — proven against a real database, not a mock.
    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn get_planning_deduplicates_a_person_who_is_both_member_and_employee() {
        let pool = make_pool().await;
        let fixture = seed_fixture(&pool).await;
        let linked_user_id = seed_employee_with_user(&pool, fixture.organization_id).await;
        seed_member(&pool, fixture.organization_id, linked_user_id).await;
        let usecase = make_usecase(pool.clone());

        let view = usecase
            .get_planning(
                fixture.organization_id,
                date_range((2026, 8, 1), (2026, 8, 7)),
            )
            .await
            .expect("get_planning must succeed");

        assert_eq!(
            view.resources.len(),
            1,
            "the linked user must appear exactly once"
        );
        assert!(matches!(
            &view.resources[0],
            PlanningResource::Employee { user_id: Some(id), .. } if *id == linked_user_id
        ));

        cleanup(
            &pool,
            fixture.organization_id,
            &[fixture.owner_id, linked_user_id],
        )
        .await;
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn get_planning_lists_a_member_only_resource_alongside_employees() {
        let pool = make_pool().await;
        let fixture = seed_fixture(&pool).await;
        let employee_id = seed_employee(&pool, fixture.organization_id).await;
        let member_only_user_id =
            seed_member_without_employee(&pool, fixture.organization_id).await;
        let usecase = make_usecase(pool.clone());

        let view = usecase
            .get_planning(
                fixture.organization_id,
                date_range((2026, 8, 1), (2026, 8, 7)),
            )
            .await
            .expect("get_planning must succeed");

        assert_eq!(view.resources.len(), 2);
        assert!(view.resources.iter().any(|r| matches!(
            r,
            PlanningResource::Employee { employee_id: id, .. } if *id == employee_id
        )));
        assert!(view.resources.iter().any(|r| matches!(
            r,
            PlanningResource::Member { user_id, .. } if *user_id == member_only_user_id
        )));

        cleanup(
            &pool,
            fixture.organization_id,
            &[fixture.owner_id, member_only_user_id],
        )
        .await;
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn get_planning_returns_work_order_and_absence_entries_in_the_window() {
        let pool = make_pool().await;
        let fixture = seed_fixture(&pool).await;
        let employee_id = seed_employee(&pool, fixture.organization_id).await;
        let now = Utc::now();
        seed_work_order(&pool, &fixture, employee_id, now, now + Duration::hours(2)).await;
        seed_absence(
            &pool,
            fixture.organization_id,
            employee_id,
            now + Duration::days(1),
            now + Duration::days(1) + Duration::hours(2),
        )
        .await;
        let usecase = make_usecase(pool.clone());

        let today = now.date_naive();
        let view = usecase
            .get_planning(
                fixture.organization_id,
                DateRange::new(today, today + Duration::days(3)).unwrap(),
            )
            .await
            .expect("get_planning must succeed");

        assert_eq!(view.entries.len(), 2);
        assert!(
            view.entries
                .iter()
                .any(|e| matches!(e, crate::PlanningEntry::WorkOrder { .. }))
        );
        assert!(
            view.entries
                .iter()
                .any(|e| matches!(e, crate::PlanningEntry::Absence { .. }))
        );

        cleanup(&pool, fixture.organization_id, &[fixture.owner_id]).await;
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn get_availability_reports_an_overlapping_absence_as_a_conflict() {
        let pool = make_pool().await;
        let fixture = seed_fixture(&pool).await;
        let employee_id = seed_employee(&pool, fixture.organization_id).await;
        let now = Utc::now();
        seed_absence(
            &pool,
            fixture.organization_id,
            employee_id,
            now,
            now + Duration::hours(4),
        )
        .await;
        let usecase = make_usecase(pool.clone());

        let window = TimeRange::new(now + Duration::hours(1), now + Duration::hours(2)).unwrap();
        let reports = usecase
            .get_availability(fixture.organization_id, window)
            .await
            .expect("get_availability must succeed");

        let report = reports
            .iter()
            .find(|r| matches!(&r.resource, PlanningResource::Employee { employee_id: id, .. } if *id == employee_id))
            .expect("the seeded employee must be in the report");

        assert!(!report.available());
        assert!(
            report
                .conflicts
                .iter()
                .any(|c| matches!(c.kind, crate::ConflictKind::Absence { .. }))
        );

        cleanup(&pool, fixture.organization_id, &[fixture.owner_id]).await;
    }
}
