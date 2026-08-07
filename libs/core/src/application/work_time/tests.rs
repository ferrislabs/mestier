#[cfg(test)]
#[allow(clippy::module_inception)]
mod tests {
    use std::sync::Arc;

    use chrono::NaiveDate;
    use common::{OrganizationId, UserId, generate_uuid_v7};
    use sqlx::PgPool;

    use crate::EmployeeId;
    use crate::application::{MestierUseCase, default_authorizer};
    use crate::domain::work_time::{
        DateRange,
        commands::{ReplaceRhythmCommand, ReplaceWorkSlotsCommand, RhythmSlotInput, WorkSlotInput},
    };
    use crate::infrastructure::realtime::{EventHub, RealtimeEventPublisher};

    fn date(y: i32, m: u32, d: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, d).unwrap()
    }

    async fn make_pool() -> PgPool {
        let url = std::env::var("DATABASE_URL")
            .expect("DATABASE_URL must be set to run work_time integration tests");
        PgPool::connect(&url).await.unwrap()
    }

    fn make_usecase(pool: PgPool) -> MestierUseCase {
        let hub = EventHub::new();
        let publisher = Arc::new(RealtimeEventPublisher::new(hub));
        MestierUseCase::new(pool, default_authorizer(), publisher)
    }

    struct Fixture {
        organization_id: OrganizationId,
        employee_id: EmployeeId,
        owner_id: UserId,
    }

    /// Seeds an owner user, an organization and an employee — the minimal
    /// graph a rhythm or work slot needs to exist.
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

        let employee_id = generate_uuid_v7();
        sqlx::query!(
            r#"INSERT INTO employees (id, org_id, last_name, hourly_rate_cents, weekly_contract_minutes)
               VALUES ($1, $2, $3, $4, $5)"#,
            employee_id,
            org_id,
            "Alice Employee",
            3500,
            2100,
        )
        .execute(pool)
        .await
        .unwrap();

        Fixture {
            organization_id: OrganizationId(org_id),
            employee_id: EmployeeId(employee_id),
            owner_id: UserId(owner_id),
        }
    }

    /// Removes everything seeded under `organization_id`, cascading to
    /// employees/employee_rhythms/employee_rhythm_slots/employee_work_slots,
    /// plus the loose user rows that outlive the organization.
    async fn cleanup(pool: &PgPool, organization_id: OrganizationId, user_ids: &[UserId]) {
        sqlx::query!(
            "DELETE FROM employee_work_slots WHERE org_id = $1",
            organization_id.0
        )
        .execute(pool)
        .await
        .ok();
        sqlx::query!(
            "DELETE FROM employee_rhythms WHERE org_id = $1",
            organization_id.0
        )
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

    fn replace_rhythm_command(
        fixture: &Fixture,
        effective_from: NaiveDate,
    ) -> ReplaceRhythmCommand {
        ReplaceRhythmCommand {
            organization_id: fixture.organization_id,
            employee_id: fixture.employee_id,
            effective_from,
            effective_to: None,
            slots: vec![RhythmSlotInput {
                weekday: 1,
                starts_minute: 480,
                ends_minute: 720,
            }],
        }
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn replace_rhythm_creates_a_first_version_when_none_exists() {
        let pool = make_pool().await;
        let fixture = seed_fixture(&pool).await;
        let usecase = make_usecase(pool.clone());

        let created = usecase
            .replace_rhythm(replace_rhythm_command(&fixture, date(2026, 1, 1)))
            .await
            .expect("replace_rhythm must succeed");

        assert_eq!(created.employee_id, fixture.employee_id);
        assert_eq!(created.effective_from, date(2026, 1, 1));
        assert_eq!(created.slots.len(), 1);

        cleanup(&pool, fixture.organization_id, &[fixture.owner_id]).await;
    }

    /// The central promise of `PUT rhythm`: calling it twice with the same
    /// `effective_from` must edit the same row in place, never accumulate a
    /// second one.
    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn replace_rhythm_editing_the_same_effective_from_replaces_in_place_without_accumulating_a_second_version()
     {
        let pool = make_pool().await;
        let fixture = seed_fixture(&pool).await;
        let usecase = make_usecase(pool.clone());

        let first = usecase
            .replace_rhythm(replace_rhythm_command(&fixture, date(2026, 1, 1)))
            .await
            .unwrap();

        let mut second_command = replace_rhythm_command(&fixture, date(2026, 1, 1));
        second_command.slots = vec![RhythmSlotInput {
            weekday: 2,
            starts_minute: 540,
            ends_minute: 1020,
        }];
        let second = usecase.replace_rhythm(second_command).await.unwrap();

        assert_eq!(
            second.id, first.id,
            "editing the same effective_from must reuse the same rhythm row"
        );
        assert_eq!(second.slots.len(), 1);
        assert_eq!(second.slots[0].weekday, 2);

        let (rhythms, _) = usecase
            .get_work_time(
                fixture.employee_id,
                DateRange::new(date(2025, 12, 1), date(2026, 2, 1)).unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(
            rhythms.len(),
            1,
            "the employee must have exactly one rhythm version, not two"
        );

        cleanup(&pool, fixture.organization_id, &[fixture.owner_id]).await;
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn replace_rhythm_with_a_later_effective_from_closes_the_open_version_and_preserves_it_as_history()
     {
        let pool = make_pool().await;
        let fixture = seed_fixture(&pool).await;
        let usecase = make_usecase(pool.clone());

        let first = usecase
            .replace_rhythm(replace_rhythm_command(&fixture, date(2026, 1, 1)))
            .await
            .unwrap();

        let second = usecase
            .replace_rhythm(replace_rhythm_command(&fixture, date(2026, 9, 1)))
            .await
            .unwrap();

        assert_ne!(second.id, first.id);
        assert_eq!(second.effective_from, date(2026, 9, 1));
        assert_eq!(second.effective_to, None);

        let (rhythms, _) = usecase
            .get_work_time(
                fixture.employee_id,
                DateRange::new(date(2026, 1, 1), date(2026, 9, 1)).unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(
            rhythms.len(),
            2,
            "the closed version must survive as history"
        );
        let closed = rhythms.iter().find(|r| r.id == first.id).unwrap();
        assert_eq!(closed.effective_to, Some(date(2026, 9, 1)));

        cleanup(&pool, fixture.organization_id, &[fixture.owner_id]).await;
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn replace_work_slots_replaces_the_window_wholesale() {
        let pool = make_pool().await;
        let fixture = seed_fixture(&pool).await;
        let usecase = make_usecase(pool.clone());

        let first_command = ReplaceWorkSlotsCommand {
            organization_id: fixture.organization_id,
            employee_id: fixture.employee_id,
            from: date(2026, 8, 1),
            to: date(2026, 8, 7),
            slots: vec![WorkSlotInput {
                work_date: date(2026, 8, 3),
                starts_minute: 480,
                ends_minute: 720,
            }],
        };
        usecase.replace_work_slots(first_command).await.unwrap();

        let second_command = ReplaceWorkSlotsCommand {
            organization_id: fixture.organization_id,
            employee_id: fixture.employee_id,
            from: date(2026, 8, 1),
            to: date(2026, 8, 7),
            slots: vec![WorkSlotInput {
                work_date: date(2026, 8, 5),
                starts_minute: 540,
                ends_minute: 1020,
            }],
        };
        let replaced = usecase.replace_work_slots(second_command).await.unwrap();

        assert_eq!(replaced.len(), 1);
        assert_eq!(replaced[0].work_date, date(2026, 8, 5));

        let (_, work_slots) = usecase
            .get_work_time(
                fixture.employee_id,
                DateRange::new(date(2026, 8, 1), date(2026, 8, 7)).unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(
            work_slots.len(),
            1,
            "the first slot must be gone after the window was replaced"
        );
        assert_eq!(work_slots[0].work_date, date(2026, 8, 5));

        cleanup(&pool, fixture.organization_id, &[fixture.owner_id]).await;
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn get_work_time_returns_overlapping_rhythm_versions_and_windowed_work_slots() {
        let pool = make_pool().await;
        let fixture = seed_fixture(&pool).await;
        let usecase = make_usecase(pool.clone());

        usecase
            .replace_rhythm(replace_rhythm_command(&fixture, date(2026, 1, 1)))
            .await
            .unwrap();
        usecase
            .replace_work_slots(ReplaceWorkSlotsCommand {
                organization_id: fixture.organization_id,
                employee_id: fixture.employee_id,
                from: date(2026, 3, 1),
                to: date(2026, 3, 7),
                slots: vec![WorkSlotInput {
                    work_date: date(2026, 3, 3),
                    starts_minute: 480,
                    ends_minute: 720,
                }],
            })
            .await
            .unwrap();
        // Outside the queried window: must not come back.
        usecase
            .replace_work_slots(ReplaceWorkSlotsCommand {
                organization_id: fixture.organization_id,
                employee_id: fixture.employee_id,
                from: date(2026, 6, 1),
                to: date(2026, 6, 7),
                slots: vec![WorkSlotInput {
                    work_date: date(2026, 6, 3),
                    starts_minute: 480,
                    ends_minute: 720,
                }],
            })
            .await
            .unwrap();

        let (rhythms, work_slots) = usecase
            .get_work_time(
                fixture.employee_id,
                DateRange::new(date(2026, 3, 1), date(2026, 3, 7)).unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(
            rhythms.len(),
            1,
            "the open rhythm version overlaps the window"
        );
        assert_eq!(work_slots.len(), 1);
        assert_eq!(work_slots[0].work_date, date(2026, 3, 3));

        cleanup(&pool, fixture.organization_id, &[fixture.owner_id]).await;
    }
}
