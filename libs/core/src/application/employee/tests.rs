#[cfg(test)]
#[allow(clippy::module_inception)]
mod tests {
    use authz::Subject;
    use chrono::Utc;
    use common::{OrganizationId, UserId, generate_uuid_v7};
    use sqlx::PgPool;

    use crate::MemberId;
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
        member_id: MemberId,
        owner_id: UserId,
    }

    /// Seeds an owner user, an organization and a plannable member with no
    /// contractual profile yet — the minimal graph `upsert_employee_profile`
    /// needs to attach one.
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
            "Alice Employee",
        )
        .execute(pool)
        .await
        .unwrap();

        Fixture {
            organization_id: OrganizationId(org_id),
            member_id: MemberId(member_id),
            owner_id: UserId(owner_id),
        }
    }

    /// Removes everything seeded under `organization_id`, cascading to
    /// employees/employee_cost_bases, plus the loose user rows that outlive
    /// the organization.
    async fn cleanup(pool: &PgPool, organization_id: OrganizationId, user_ids: &[UserId]) {
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
            "DELETE FROM organizations WHERE id = $1",
            organization_id.0,
        )
        .await;
        for uid in user_ids {
            purge(pool, "DELETE FROM users WHERE id = $1", uid.0).await;
        }
    }

    struct CostBasisRow {
        effective_from: chrono::NaiveDate,
        effective_to: Option<chrono::NaiveDate>,
        hourly_rate_cents: Option<i32>,
    }

    async fn open_cost_basis(pool: &PgPool, employee_id: uuid::Uuid) -> CostBasisRow {
        let row = sqlx::query!(
            r#"SELECT effective_from, effective_to, hourly_rate_cents
               FROM employee_cost_bases WHERE employee_id = $1 AND effective_to IS NULL"#,
            employee_id,
        )
        .fetch_one(pool)
        .await
        .unwrap();

        CostBasisRow {
            effective_from: row.effective_from,
            effective_to: row.effective_to,
            hourly_rate_cents: row.hourly_rate_cents,
        }
    }

    async fn cost_basis_count(pool: &PgPool, employee_id: uuid::Uuid) -> i64 {
        sqlx::query_scalar!(
            r#"SELECT COUNT(*) AS "count!" FROM employee_cost_bases WHERE employee_id = $1"#,
            employee_id,
        )
        .fetch_one(pool)
        .await
        .unwrap()
    }

    /// Attaching a brand new profile must open a first cost basis version
    /// dated today, not leave `employee_cost_bases` empty until somebody
    /// changes the rate later — #301's per-task join has nothing to read
    /// otherwise.
    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn upsert_employee_profile_opens_a_first_cost_basis_version_dated_today() {
        let pool = make_pool().await;
        let fixture = seed_fixture(&pool).await;
        let usecase = make_usecase(pool.clone());

        let employee = usecase
            .upsert_employee_profile(
                Subject::system(),
                fixture.member_id,
                Some(3500),
                false,
                None,
                2100,
            )
            .await
            .expect("upsert_employee_profile must succeed");

        let basis = open_cost_basis(&pool, employee.id.0).await;
        assert_eq!(basis.effective_from, Utc::now().date_naive());
        assert_eq!(basis.effective_to, None);
        assert_eq!(basis.hourly_rate_cents, Some(3500));

        cleanup(&pool, fixture.organization_id, &[fixture.owner_id]).await;
    }

    /// Calling the upsert twice the same day (e.g. two form submissions, or a
    /// typo corrected minutes later) must edit the open version in place,
    /// never accumulate a second row for the same day —
    /// `uq_employee_cost_bases_open_version` would reject a second open
    /// version anyway, and a service that could hit that constraint in
    /// normal use is not a design.
    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn upsert_employee_profile_twice_the_same_day_edits_the_open_version_in_place() {
        let pool = make_pool().await;
        let fixture = seed_fixture(&pool).await;
        let usecase = make_usecase(pool.clone());

        let employee = usecase
            .upsert_employee_profile(
                Subject::system(),
                fixture.member_id,
                Some(3500),
                false,
                None,
                2100,
            )
            .await
            .unwrap();

        usecase
            .upsert_employee_profile(
                Subject::system(),
                fixture.member_id,
                Some(4200),
                false,
                None,
                2100,
            )
            .await
            .unwrap();

        assert_eq!(
            cost_basis_count(&pool, employee.id.0).await,
            1,
            "must still have exactly one cost basis version"
        );
        let basis = open_cost_basis(&pool, employee.id.0).await;
        assert_eq!(basis.hourly_rate_cents, Some(4200));

        cleanup(&pool, fixture.organization_id, &[fixture.owner_id]).await;
    }

    /// The `employees` columns are a projection of the open cost basis
    /// version — they must always agree, since profitability's fast paths
    /// and the cost history read the same fact from two tables.
    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn upsert_employee_profile_keeps_the_employees_columns_in_sync_with_the_open_version() {
        let pool = make_pool().await;
        let fixture = seed_fixture(&pool).await;
        let usecase = make_usecase(pool.clone());

        let employee = usecase
            .upsert_employee_profile(
                Subject::system(),
                fixture.member_id,
                Some(3500),
                false,
                None,
                2100,
            )
            .await
            .unwrap();

        let basis = open_cost_basis(&pool, employee.id.0).await;
        assert_eq!(employee.hourly_rate_cents, basis.hourly_rate_cents);

        cleanup(&pool, fixture.organization_id, &[fixture.owner_id]).await;
    }
}
