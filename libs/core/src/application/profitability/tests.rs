#[cfg(test)]
#[allow(clippy::module_inception)]
mod tests {
    use chrono::{Duration, NaiveDate, Utc};
    use common::{OrganizationId, UserId, generate_uuid_v7};
    use sqlx::PgPool;

    use crate::application::test_support::{dev_pool, purge};
    use crate::application::{MestierUseCase, default_authorizer};
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

    /// Seeds an owner, an organization, a member with an hourly profile, and
    /// two cost basis versions: a closed one at 30 €/h effective from a month
    /// ago through today, and the open one at 40 €/h from today onward — a
    /// raise that took effect today, exactly the shape #301 exists to cost
    /// correctly. Two tasks are planted, one under each version.
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

        let member_id = generate_uuid_v7();
        sqlx::query!(
            r#"INSERT INTO organization_members (id, organization_id, last_name)
               VALUES ($1, $2, $3)"#,
            member_id,
            org_id,
            "Raised Employee",
        )
        .execute(pool)
        .await
        .unwrap();

        let employee_id = generate_uuid_v7();
        sqlx::query!(
            r#"INSERT INTO employees (id, org_id, member_id, hourly_rate_cents, weekly_contract_minutes)
               VALUES ($1, $2, $3, $4, $5)"#,
            employee_id,
            org_id,
            member_id,
            4_000,
            2_100,
        )
        .execute(pool)
        .await
        .unwrap();

        let today = Utc::now().date_naive();
        let a_month_ago = today - Duration::days(30);

        sqlx::query!(
            r#"INSERT INTO employee_cost_bases (id, org_id, employee_id, effective_from, effective_to, hourly_rate_cents, weekly_contract_minutes)
               VALUES ($1, $2, $3, $4, $5, $6, $7)"#,
            generate_uuid_v7(),
            org_id,
            employee_id,
            a_month_ago,
            today,
            3_000,
            2_100,
        )
        .execute(pool)
        .await
        .unwrap();

        sqlx::query!(
            r#"INSERT INTO employee_cost_bases (id, org_id, employee_id, effective_from, effective_to, hourly_rate_cents, weekly_contract_minutes)
               VALUES ($1, $2, $3, $4, $5, $6, $7)"#,
            generate_uuid_v7(),
            org_id,
            employee_id,
            today,
            Option::<NaiveDate>::None,
            4_000,
            2_100,
        )
        .execute(pool)
        .await
        .unwrap();

        // Two hours, ten days ago: under the closed, 30 €/h version.
        seed_task(
            pool,
            org_id,
            member_id,
            (today - Duration::days(10))
                .and_hms_opt(9, 0, 0)
                .unwrap()
                .and_utc(),
            (today - Duration::days(10))
                .and_hms_opt(11, 0, 0)
                .unwrap()
                .and_utc(),
        )
        .await;

        // Two hours, today: under the open, 40 €/h version.
        seed_task(
            pool,
            org_id,
            member_id,
            today.and_hms_opt(9, 0, 0).unwrap().and_utc(),
            today.and_hms_opt(11, 0, 0).unwrap().and_utc(),
        )
        .await;

        Fixture {
            organization_id: OrganizationId(org_id),
            owner_id: UserId(owner_id),
        }
    }

    async fn seed_task(
        pool: &PgPool,
        org_id: uuid::Uuid,
        member_id: uuid::Uuid,
        starts_at: chrono::DateTime<Utc>,
        ends_at: chrono::DateTime<Utc>,
    ) {
        let task_id = generate_uuid_v7();
        sqlx::query!(
            r#"INSERT INTO tasks (id, org_id, starts_at, ends_at, all_day, status, title)
               VALUES ($1, $2, $3, $4, false, 'PLANNED', 'Chantier')"#,
            task_id,
            org_id,
            starts_at,
            ends_at,
        )
        .execute(pool)
        .await
        .unwrap();

        sqlx::query!(
            r#"INSERT INTO task_assignments (id, org_id, task_id, member_id)
               VALUES ($1, $2, $3, $4)"#,
            generate_uuid_v7(),
            org_id,
            task_id,
            member_id,
        )
        .execute(pool)
        .await
        .unwrap();
    }

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
            "DELETE FROM employee_cost_bases WHERE org_id = $1",
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
            "DELETE FROM organization_members WHERE organization_id = $1",
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

    /// The regression #301 exists to close: a task planned under the old
    /// rate keeps its old cost after a raise takes effect, and a task
    /// planned under the new rate gets the new one — through the real
    /// Postgres adapter, not the pure domain function.
    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn a_report_covering_both_versions_costs_each_task_at_the_rate_that_applied_that_day() {
        let pool = make_pool().await;
        let fixture = seed_fixture(&pool).await;
        let usecase = make_usecase(pool.clone());
        let today = Utc::now().date_naive();

        let old_period_report = usecase
            .profitability_report(
                fixture.organization_id,
                today - Duration::days(10),
                today - Duration::days(10),
            )
            .await
            .expect("the report must succeed");
        assert_eq!(
            old_period_report.members[0].labour_cost_cents, 6_000,
            "two hours at the 30 €/h version that covered that day"
        );

        let new_period_report = usecase
            .profitability_report(fixture.organization_id, today, today)
            .await
            .expect("the report must succeed");
        assert_eq!(
            new_period_report.members[0].labour_cost_cents, 8_000,
            "two hours at the 40 €/h version that covers today"
        );

        // The whole point: querying a period that spans the raise must not
        // recompute the old day at the new rate.
        let spanning_report = usecase
            .profitability_report(fixture.organization_id, today - Duration::days(10), today)
            .await
            .expect("the report must succeed");
        assert_eq!(
            spanning_report.members[0].labour_cost_cents,
            6_000 + 8_000,
            "each day must keep the rate that applied on it"
        );

        cleanup(&pool, fixture.organization_id, &[fixture.owner_id]).await;
    }
}
