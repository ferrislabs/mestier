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
        seed_task_with_status(
            pool,
            org_id,
            member_id,
            Some(starts_at),
            Some(ends_at),
            "PLANNED",
        )
        .await;
    }

    /// The general form: a status of the caller's choosing, and a window that
    /// may be absent entirely. `starts_at`/`ends_at` are passed as a pair
    /// because `chk_tasks_dates_both_or_neither` allows nothing else — both,
    /// or neither.
    async fn seed_task_with_status(
        pool: &PgPool,
        org_id: uuid::Uuid,
        member_id: uuid::Uuid,
        starts_at: Option<chrono::DateTime<Utc>>,
        ends_at: Option<chrono::DateTime<Utc>>,
        status: &str,
    ) {
        let task_id = generate_uuid_v7();
        // `($5::text)::task_status` rather than a bare `$5::task_status`: the
        // inner cast is what lets sqlx infer the parameter as `text` and keep
        // this a compile-time-checked `query!` while the status stays a
        // caller-chosen `&str`.
        sqlx::query!(
            r#"INSERT INTO tasks (id, org_id, starts_at, ends_at, all_day, status, title)
               VALUES ($1, $2, $3, $4, false, ($5::text)::task_status, 'Chantier')"#,
            task_id,
            org_id,
            starts_at,
            ends_at,
            status,
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

    /// Both halves of the orthogonality rule, through the real adapter, where
    /// the window filter actually lives.
    ///
    /// The window is the filter: two undated tasks — one `BACKLOG`, one
    /// `IN_PROGRESS` — contribute nothing, because a cost is minutes times a
    /// rate and they have no minutes. The status is not the filter: a dated
    /// `BACKLOG` task costs exactly what the dated `PLANNED` one beside it
    /// costs, because dating something is committing time to it whatever the
    /// work's progress.
    ///
    /// And the skipped rows take nothing with them: the dated tasks' own cost
    /// comes back whole.
    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn undated_tasks_cost_nothing_while_a_dated_backlog_task_costs_like_any_other() {
        let pool = make_pool().await;
        let fixture = seed_fixture(&pool).await;
        let usecase = make_usecase(pool.clone());
        let today = Utc::now().date_naive();

        let member_id: uuid::Uuid = sqlx::query_scalar!(
            r#"SELECT id FROM organization_members WHERE organization_id = $1"#,
            fixture.organization_id.0,
        )
        .fetch_one(&pool)
        .await
        .unwrap();

        // Before: the fixture's one dated task on today, two hours at 40 €/h.
        let before = usecase
            .profitability_report(fixture.organization_id, today, today)
            .await
            .expect("the report must succeed");
        assert_eq!(before.members[0].labour_cost_cents, 8_000);

        seed_task_with_status(
            &pool,
            fixture.organization_id.0,
            member_id,
            None,
            None,
            "BACKLOG",
        )
        .await;
        // Undatedness is what excludes a task, never its column: an
        // `IN_PROGRESS` task with no window is just as absent as a backlog one.
        seed_task_with_status(
            &pool,
            fixture.organization_id.0,
            member_id,
            None,
            None,
            "IN_PROGRESS",
        )
        .await;

        let with_undated = usecase
            .profitability_report(fixture.organization_id, today, today)
            .await
            .expect("the report must succeed");
        assert_eq!(
            with_undated.members[0].labour_cost_cents, 8_000,
            "an undated task has no minutes to cost, and takes none away from the dated one"
        );
        assert_eq!(
            with_undated.members[0].planned_minutes, 120,
            "no invented window — not now(), not a zero-length range — reached the total"
        );

        seed_task_with_status(
            &pool,
            fixture.organization_id.0,
            member_id,
            Some(today.and_hms_opt(14, 0, 0).unwrap().and_utc()),
            Some(today.and_hms_opt(16, 0, 0).unwrap().and_utc()),
            "BACKLOG",
        )
        .await;

        let with_dated_backlog = usecase
            .profitability_report(fixture.organization_id, today, today)
            .await
            .expect("the report must succeed");
        assert_eq!(
            with_dated_backlog.members[0].labour_cost_cents,
            8_000 + 8_000,
            "a dated BACKLOG task costs its two hours like any other dated task — \
             the status is not a visibility filter"
        );

        cleanup(&pool, fixture.organization_id, &[fixture.owner_id]).await;
    }
}
