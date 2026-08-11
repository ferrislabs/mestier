#[cfg(test)]
#[allow(clippy::module_inception)]
mod tests {

    use chrono::{Duration, Utc};
    use common::{OrganizationId, UserId, generate_uuid_v7};
    use sqlx::PgPool;

    use crate::application::{MestierUseCase, default_authorizer};
    use crate::domain::absence::commands::{CreateAbsenceCommand, PatchAbsenceCommand};
    use crate::infrastructure::realtime::EventHub;
    use crate::{AbsenceId, AbsenceKind, EmployeeId, MemberId};

    async fn make_pool() -> PgPool {
        let url = std::env::var("DATABASE_URL")
            .expect("DATABASE_URL must be set to run absence integration tests");
        PgPool::connect(&url).await.unwrap()
    }

    fn make_usecase(pool: PgPool) -> MestierUseCase {
        MestierUseCase::new(pool, default_authorizer(), EventHub::new())
    }

    struct Fixture {
        organization_id: OrganizationId,
        member_id: MemberId,
        #[allow(dead_code)]
        employee_id: EmployeeId,
        owner_id: UserId,
    }

    /// Seeds an owner user, an organization and an employee — the minimal
    /// graph an absence needs to exist.
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

        let employee_id = generate_uuid_v7();
        sqlx::query!(
            r#"INSERT INTO employees (id, org_id, member_id, hourly_rate_cents, weekly_contract_minutes)
               VALUES ($1, $2, $3, $4, $5)"#,
            employee_id,
            org_id,
            member_id,
            3500,
            2100,
        )
        .execute(pool)
        .await
        .unwrap();

        Fixture {
            organization_id: OrganizationId(org_id),
            member_id: MemberId(member_id),
            employee_id: EmployeeId(employee_id),
            owner_id: UserId(owner_id),
        }
    }

    /// Removes everything seeded under `organization_id`, cascading to
    /// employees/absences, plus the loose user rows that outlive
    /// the organization.
    async fn cleanup(pool: &PgPool, organization_id: OrganizationId, user_ids: &[UserId]) {
        sqlx::query!("DELETE FROM absences WHERE org_id = $1", organization_id.0)
            .execute(pool)
            .await
            .ok();
        sqlx::query!("DELETE FROM employees WHERE org_id = $1", organization_id.0)
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

    fn create_command(fixture: &Fixture) -> CreateAbsenceCommand {
        let now = Utc::now();
        CreateAbsenceCommand {
            organization_id: fixture.organization_id,
            member_id: fixture.member_id,
            kind: AbsenceKind::Leave,
            starts_at: now,
            ends_at: now + Duration::hours(2),
            all_day: false,
            note: Some("Congés payés".to_owned()),
        }
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn create_absence_persists_and_is_retrievable() {
        let pool = make_pool().await;
        let fixture = seed_fixture(&pool).await;
        let usecase = make_usecase(pool.clone());

        let created = usecase
            .create_absence(create_command(&fixture))
            .await
            .expect("create_absence must succeed");

        assert_eq!(created.kind, AbsenceKind::Leave);
        assert_eq!(created.member_id, fixture.member_id);

        let fetched = usecase
            .get_absence(created.id)
            .await
            .expect("get_absence must succeed");
        assert_eq!(fetched.id, created.id);
        assert_eq!(fetched.note.as_deref(), Some("Congés payés"));

        cleanup(&pool, fixture.organization_id, &[fixture.owner_id]).await;
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn list_absences_returns_only_the_organizations_own() {
        let pool = make_pool().await;
        let fixture_a = seed_fixture(&pool).await;
        let fixture_b = seed_fixture(&pool).await;
        let usecase = make_usecase(pool.clone());

        usecase
            .create_absence(create_command(&fixture_a))
            .await
            .unwrap();
        usecase
            .create_absence(create_command(&fixture_b))
            .await
            .unwrap();

        let (items, total) = usecase
            .list_absences(fixture_a.organization_id, 20, 0)
            .await
            .expect("list_absences must succeed");

        assert_eq!(total, 1);
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].organization_id, fixture_a.organization_id);

        cleanup(&pool, fixture_a.organization_id, &[fixture_a.owner_id]).await;
        cleanup(&pool, fixture_b.organization_id, &[fixture_b.owner_id]).await;
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn patch_absence_reschedules_and_changes_kind() {
        let pool = make_pool().await;
        let fixture = seed_fixture(&pool).await;
        let usecase = make_usecase(pool.clone());

        let created = usecase
            .create_absence(create_command(&fixture))
            .await
            .unwrap();

        let new_starts_at = created.starts_at + Duration::days(1);
        let new_ends_at = created.ends_at + Duration::days(1);

        let mut patch = PatchAbsenceCommand::new(created.id);
        patch.starts_at = Some(new_starts_at);
        patch.ends_at = Some(new_ends_at);
        patch.kind = Some(AbsenceKind::Sick);
        patch.note = Some(None);

        let updated = usecase
            .patch_absence(patch)
            .await
            .expect("patch_absence must succeed");

        assert_eq!(updated.starts_at, new_starts_at);
        assert_eq!(updated.ends_at, new_ends_at);
        assert_eq!(updated.kind, AbsenceKind::Sick);
        assert!(updated.note.is_none());

        cleanup(&pool, fixture.organization_id, &[fixture.owner_id]).await;
    }

    /// Proves the promise a `deleted_at` column makes: soft-deleting an
    /// absence removes it from reads (`get_absence`, `list_absences`) while
    /// the row itself survives in the table, deleted_at populated.
    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn soft_delete_absence_hides_it_from_reads_but_keeps_the_row_in_the_database() {
        let pool = make_pool().await;
        let fixture = seed_fixture(&pool).await;
        let usecase = make_usecase(pool.clone());

        let created = usecase
            .create_absence(create_command(&fixture))
            .await
            .unwrap();

        usecase
            .soft_delete_absence(created.id)
            .await
            .expect("soft_delete_absence must succeed");

        let err = usecase
            .get_absence(created.id)
            .await
            .expect_err("a soft-deleted absence must not be gettable");
        assert!(matches!(err, common::CoreError::NotFound));

        let (items, total) = usecase
            .list_absences(fixture.organization_id, 20, 0)
            .await
            .unwrap();
        assert_eq!(total, 0);
        assert!(items.is_empty());

        let row = sqlx::query!(
            r#"SELECT deleted_at FROM absences WHERE id = $1"#,
            created.id.0,
        )
        .fetch_one(&pool)
        .await
        .expect("the row must still physically exist after a soft delete");
        assert!(
            row.deleted_at.is_some(),
            "deleted_at must be populated by the soft delete"
        );

        let missing = usecase
            .soft_delete_absence(AbsenceId(generate_uuid_v7()))
            .await
            .expect_err("deleting an unknown absence must fail");
        assert!(matches!(missing, common::CoreError::NotFound));

        cleanup(&pool, fixture.organization_id, &[fixture.owner_id]).await;
    }
}
