#[cfg(test)]
#[allow(clippy::module_inception)]
mod tests {
    use chrono::{NaiveDate, NaiveTime, Weekday};
    use chrono_tz::Europe;
    use common::{OrganizationId, UserId, generate_uuid_v7};
    use sqlx::PgPool;

    use crate::MemberId;
    use crate::application::test_support::{dev_pool, purge};
    use crate::application::{MestierUseCase, default_authorizer};
    use crate::domain::task::commands::PatchTaskCommand;
    use crate::domain::task_recurrence::RecurrenceRule;
    use crate::domain::task_recurrence::commands::CreateTaskRecurrenceCommand;
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
            "Alice Membre",
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
            "DELETE FROM task_recurrences WHERE org_id = $1",
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

    fn daily_command(fixture: &Fixture, starts_on: NaiveDate) -> CreateTaskRecurrenceCommand {
        CreateTaskRecurrenceCommand {
            organization_id: fixture.organization_id,
            rule: RecurrenceRule::Daily,
            starts_on,
            ends_on: None,
            timezone: Europe::Paris,
            start_time: NaiveTime::from_hms_opt(9, 0, 0).unwrap(),
            duration_minutes: 60,
            all_day: false,
            title: "Réunion hebdo".to_owned(),
            description: None,
            blocks_availability: true,
            customer_id: None,
            customer_context_id: None,
            project_id: None,
            assignee_member_ids: vec![fixture.member_id],
        }
    }

    async fn task_count(pool: &PgPool, recurrence_id: uuid::Uuid) -> i64 {
        sqlx::query_scalar!(
            "SELECT COUNT(*) FROM tasks WHERE recurrence_id = $1",
            recurrence_id,
        )
        .fetch_one(pool)
        .await
        .unwrap()
        .unwrap_or(0)
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn creating_a_daily_recurrence_materializes_tasks_up_to_the_default_horizon() {
        let pool = make_pool().await;
        let fixture = seed_fixture(&pool).await;
        let usecase = make_usecase(pool.clone());

        // `target_horizon` fills to `horizon_days` out from *today*, or
        // `starts_on`, whichever is later — a fixed past date would make the
        // assertion below drift by however many days have passed since it
        // was written, so `starts_on` is today's date, always the later one.
        // "Today" has to be read in the recurrence's own timezone (`daily_command`
        // sets `Europe::Paris`), the same one `target_horizon` converts `now`
        // into: reading it in UTC instead drifts by a day for part of every
        // day Paris's calendar date has already rolled over ahead of UTC's.
        let created = usecase
            .create_task_recurrence(daily_command(
                &fixture,
                chrono::Utc::now()
                    .with_timezone(&Europe::Paris)
                    .date_naive(),
            ))
            .await
            .unwrap();

        assert_eq!(
            created.horizon_filled_to,
            created.starts_on
                + chrono::Duration::days(
                    crate::domain::task_recurrence::service::DEFAULT_HORIZON_DAYS
                )
        );

        // One task per day from starts_on to horizon_filled_to, inclusive.
        let expected = (created.horizon_filled_to - created.starts_on).num_days() + 1;
        assert_eq!(task_count(&pool, created.id.0).await, expected);

        let one_task = sqlx::query!(
            r#"SELECT t.id, t.occurrence_date, a.member_id
               FROM tasks t
               LEFT JOIN task_assignments a ON a.task_id = t.id
               WHERE t.recurrence_id = $1
               ORDER BY t.occurrence_date ASC
               LIMIT 1"#,
            created.id.0,
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(one_task.occurrence_date, Some(created.starts_on));
        assert_eq!(one_task.member_id, fixture.member_id.0);

        cleanup(&pool, fixture.organization_id, &[fixture.owner_id]).await;
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn creating_a_weekly_recurrence_only_materializes_the_chosen_weekdays() {
        let pool = make_pool().await;
        let fixture = seed_fixture(&pool).await;
        let usecase = make_usecase(pool.clone());

        let mut command = daily_command(&fixture, NaiveDate::from_ymd_opt(2026, 8, 1).unwrap());
        command.rule = RecurrenceRule::Weekly {
            weekdays: vec![Weekday::Tue],
        };
        command.ends_on = Some(NaiveDate::from_ymd_opt(2026, 8, 31).unwrap());

        let created = usecase.create_task_recurrence(command).await.unwrap();

        // Every Tuesday in August 2026: the 4th, 11th, 18th, 25th.
        assert_eq!(task_count(&pool, created.id.0).await, 4);

        cleanup(&pool, fixture.organization_id, &[fixture.owner_id]).await;
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn an_unknown_assignee_fails_creation_and_materializes_nothing() {
        let pool = make_pool().await;
        let fixture = seed_fixture(&pool).await;
        let usecase = make_usecase(pool.clone());

        let mut command = daily_command(&fixture, NaiveDate::from_ymd_opt(2026, 8, 24).unwrap());
        command.assignee_member_ids = vec![MemberId(uuid::Uuid::new_v4())];

        let err = usecase.create_task_recurrence(command).await.unwrap_err();
        assert!(matches!(err, common::CoreError::NotFound));

        let count: i64 = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM task_recurrences WHERE org_id = $1",
            fixture.organization_id.0,
        )
        .fetch_one(&pool)
        .await
        .unwrap()
        .unwrap_or(0);
        assert_eq!(count, 0, "nothing must be left behind by a failed creation");

        cleanup(&pool, fixture.organization_id, &[fixture.owner_id]).await;
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn editing_one_occurrence_detaches_it_but_keeps_its_own_life() {
        let pool = make_pool().await;
        let fixture = seed_fixture(&pool).await;
        let usecase = make_usecase(pool.clone());

        let created = usecase
            .create_task_recurrence(daily_command(
                &fixture,
                NaiveDate::from_ymd_opt(2026, 8, 24).unwrap(),
            ))
            .await
            .unwrap();

        let occurrence_id: uuid::Uuid = sqlx::query_scalar!(
            "SELECT id FROM tasks WHERE recurrence_id = $1 ORDER BY occurrence_date ASC LIMIT 1",
            created.id.0,
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        let occurrence_id = crate::TaskId(occurrence_id);

        let mut patch = PatchTaskCommand::new(occurrence_id);
        patch.title = Some("Réunion déplacée".to_owned());

        let patched = usecase.patch_task(patch).await.unwrap();

        assert!(
            patched.recurrence_id.is_none(),
            "editing an occurrence must detach it from its series"
        );
        assert_eq!(patched.title, "Réunion déplacée");
        assert!(
            patched.occurrence_date.is_some(),
            "the occurrence date is kept even once detached"
        );

        // The other occurrences are untouched.
        let remaining = task_count(&pool, created.id.0).await;
        let expected = (created.horizon_filled_to - created.starts_on).num_days();
        assert_eq!(
            remaining, expected,
            "only the edited occurrence left the series"
        );

        cleanup(&pool, fixture.organization_id, &[fixture.owner_id]).await;
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn get_and_list_recurrences_scope_by_organization() {
        let pool = make_pool().await;
        let mine = seed_fixture(&pool).await;
        let theirs = seed_fixture(&pool).await;
        let usecase = make_usecase(pool.clone());

        let created = usecase
            .create_task_recurrence(daily_command(
                &mine,
                NaiveDate::from_ymd_opt(2026, 8, 24).unwrap(),
            ))
            .await
            .unwrap();
        usecase
            .create_task_recurrence(daily_command(
                &theirs,
                NaiveDate::from_ymd_opt(2026, 8, 24).unwrap(),
            ))
            .await
            .unwrap();

        let fetched = usecase.get_task_recurrence(created.id).await.unwrap();
        assert_eq!(fetched.id, created.id);

        let listed = usecase
            .list_task_recurrences(mine.organization_id)
            .await
            .unwrap();
        assert_eq!(listed.len(), 1);
        assert_eq!(listed[0].id, created.id);

        cleanup(&pool, mine.organization_id, &[mine.owner_id]).await;
        cleanup(&pool, theirs.organization_id, &[theirs.owner_id]).await;
    }

    // -- ensure_recurrence_horizon_runs / the horizon-extension pass (#293) --
    //
    // Its own database (`automation_pool`), not `dev_pool`: `ensure_recurrence_
    // horizon_runs` creates `automation.run` rows, and `run_engine_pass` claims
    // every *due* run system-wide — see `application::automation::run`'s own
    // note on why that suite needs an empty queue nobody else fills. The
    // fixtures below live in the automation scratch database too, so a
    // recurrence's org/member rows are seeded and torn down there directly
    // rather than shared with the `dev_pool` fixtures above.

    mod horizon_extension_tests {
        use std::time::Duration as StdDuration;

        use chrono::{Duration as ChronoDuration, Utc};

        use super::*;
        use crate::application::test_support::automation_pool;
        use crate::infrastructure::automation::connectors::ConnectorRegistry;
        use crate::infrastructure::automation::postgres::run::RUN_CLAIM_LOCK;
        use crate::infrastructure::automation::worker::WorkerSchedule;

        async fn make_automation_pool() -> PgPool {
            automation_pool().await
        }

        fn generous_schedule() -> WorkerSchedule {
            WorkerSchedule {
                interval: StdDuration::from_secs(5),
                batch: 100,
                per_org: 100,
                claim_timeout: StdDuration::from_secs(300),
                max_steps_per_slice: 10_000,
                max_slice_duration: StdDuration::from_secs(120),
            }
        }

        async fn run_count_for_org(pool: &PgPool, org_id: OrganizationId) -> i64 {
            sqlx::query_scalar!(
                "SELECT COUNT(*) FROM automation.run WHERE org_id = $1",
                org_id.0,
            )
            .fetch_one(pool)
            .await
            .unwrap()
            .unwrap_or(0)
        }

        async fn total_run_count(pool: &PgPool) -> i64 {
            sqlx::query_scalar!("SELECT COUNT(*) FROM automation.run")
                .fetch_one(pool)
                .await
                .unwrap()
                .unwrap_or(0)
        }

        async fn push_horizon_filled_to(pool: &PgPool, recurrence_id: uuid::Uuid, date: NaiveDate) {
            sqlx::query!(
                "UPDATE task_recurrences SET horizon_filled_to = $2 WHERE id = $1",
                recurrence_id,
                date,
            )
            .execute(pool)
            .await
            .unwrap();
        }

        /// The acceptance criterion the whole scheduling half of #293 rests
        /// on: an organization that has never created a recurrence must
        /// never produce a run. Asserted against the whole queue, not merely
        /// "no run for my org" — this suite has a history of only passing on
        /// a queue somebody else's test already filled (see
        /// `application::automation::run`'s own tests for the same
        /// discipline).
        #[tokio::test]
        #[ignore = "requires live postgres"]
        async fn an_organization_with_no_recurrences_produces_no_run() {
            let pool = make_automation_pool().await;
            let fixture = seed_fixture(&pool).await;
            let usecase = make_usecase(pool.clone());

            let created = usecase.ensure_recurrence_horizon_runs().await.unwrap();

            assert_eq!(created, 0);
            assert_eq!(total_run_count(&pool).await, 0);

            cleanup(&pool, fixture.organization_id, &[fixture.owner_id]).await;
        }

        /// A recurrence whose watermark is comfortably far in the future
        /// (freshly created, filled to the full default horizon) does not
        /// need a run yet either.
        #[tokio::test]
        #[ignore = "requires live postgres"]
        async fn a_freshly_created_recurrence_does_not_need_a_run_yet() {
            let pool = make_automation_pool().await;
            let fixture = seed_fixture(&pool).await;
            let usecase = make_usecase(pool.clone());
            usecase
                .create_task_recurrence(daily_command(
                    &fixture,
                    NaiveDate::from_ymd_opt(2026, 8, 24).unwrap(),
                ))
                .await
                .unwrap();

            let created = usecase.ensure_recurrence_horizon_runs().await.unwrap();

            assert_eq!(created, 0);
            assert_eq!(run_count_for_org(&pool, fixture.organization_id).await, 0);

            cleanup(&pool, fixture.organization_id, &[fixture.owner_id]).await;
        }

        /// The core loop: a recurrence whose horizon is getting close to
        /// today gets exactly one run, running it advances the watermark and
        /// materializes the missing occurrences, and asking again right
        /// after does not queue a second run for the same, now-satisfied,
        /// need.
        #[tokio::test]
        #[ignore = "requires live postgres"]
        async fn a_due_recurrence_gets_one_run_that_extends_it_and_stops_asking() {
            let _guard = RUN_CLAIM_LOCK.lock().await;
            let pool = make_automation_pool().await;
            let fixture = seed_fixture(&pool).await;
            let usecase = make_usecase(pool.clone());
            let created_recurrence = usecase
                .create_task_recurrence(daily_command(
                    &fixture,
                    NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
                ))
                .await
                .unwrap();
            let today = Utc::now().date_naive();
            // Simulates time passing: the horizon is now only 10 days out,
            // well inside the refill trigger window.
            push_horizon_filled_to(
                &pool,
                created_recurrence.id.0,
                today + ChronoDuration::days(10),
            )
            .await;

            let created = usecase.ensure_recurrence_horizon_runs().await.unwrap();
            assert_eq!(created, 1, "exactly one run for the one due recurrence");
            assert_eq!(run_count_for_org(&pool, fixture.organization_id).await, 1);

            // Asking again before the run is worked must not queue a second
            // one for the same still-pending need.
            let created_again = usecase.ensure_recurrence_horizon_runs().await.unwrap();
            assert_eq!(created_again, 0);
            assert_eq!(run_count_for_org(&pool, fixture.organization_id).await, 1);

            let connectors = ConnectorRegistry::new(usecase.clone());
            usecase
                .run_engine_pass(&connectors, "worker-1", generous_schedule())
                .await
                .unwrap();

            let after = usecase
                .get_task_recurrence(created_recurrence.id)
                .await
                .unwrap();
            assert!(
                after.horizon_filled_to > today + ChronoDuration::days(10),
                "the watermark actually moved: {:?}",
                after.horizon_filled_to
            );

            let task_count: i64 = sqlx::query_scalar!(
                "SELECT COUNT(*) FROM tasks WHERE recurrence_id = $1 AND occurrence_date > $2",
                created_recurrence.id.0,
                today + ChronoDuration::days(10),
            )
            .fetch_one(&pool)
            .await
            .unwrap()
            .unwrap_or(0);
            assert!(task_count > 0, "new occurrences were materialized");

            // Now comfortably filled again: no further run is queued.
            let settled = usecase.ensure_recurrence_horizon_runs().await.unwrap();
            assert_eq!(settled, 0);

            cleanup(&pool, fixture.organization_id, &[fixture.owner_id]).await;
        }

        /// Idempotency at the SQL layer: running the extension twice over
        /// the same range — the scenario a retried or double-claimed run
        /// produces — never creates a duplicate occurrence, thanks to
        /// `insert_occurrence_if_absent`'s `ON CONFLICT ... DO NOTHING`.
        #[tokio::test]
        #[ignore = "requires live postgres"]
        async fn extending_the_same_range_twice_materializes_nothing_the_second_time() {
            let pool = make_automation_pool().await;
            let fixture = seed_fixture(&pool).await;
            let usecase = make_usecase(pool.clone());
            let today = Utc::now().date_naive();
            let created_recurrence = usecase
                .create_task_recurrence(daily_command(&fixture, today))
                .await
                .unwrap();
            // Creation already filled [today, today + 60]. Rewinding the
            // watermark to `today` and deleting the rows it would otherwise
            // find already there is what makes the first pass below have
            // real work to do, rather than finding the whole range already
            // filled from creation and never reaching `insert_occurrence_if_absent`
            // at all.
            push_horizon_filled_to(&pool, created_recurrence.id.0, today).await;
            sqlx::query!(
                "DELETE FROM tasks WHERE recurrence_id = $1 AND occurrence_date > $2",
                created_recurrence.id.0,
                today,
            )
            .execute(&pool)
            .await
            .unwrap();

            let first = usecase
                .extend_recurrence_horizons_for_organization(fixture.organization_id)
                .await
                .unwrap();
            assert!(first > 0, "the deleted dates are re-materialized");

            // Force the watermark back down over the exact same range, this
            // time without deleting anything: every date is already there.
            push_horizon_filled_to(&pool, created_recurrence.id.0, today).await;

            let second = usecase
                .extend_recurrence_horizons_for_organization(fixture.organization_id)
                .await
                .unwrap();
            assert_eq!(
                second, 0,
                "every date in the range was already filled by the first pass"
            );

            cleanup(&pool, fixture.organization_id, &[fixture.owner_id]).await;
        }

        /// `ends_on` in the past stops a recurrence from being visited at
        /// all, even when its watermark is far behind.
        #[tokio::test]
        #[ignore = "requires live postgres"]
        async fn a_recurrence_whose_ends_on_has_passed_produces_no_run() {
            let pool = make_automation_pool().await;
            let fixture = seed_fixture(&pool).await;
            let usecase = make_usecase(pool.clone());
            let today = Utc::now().date_naive();

            let mut command = daily_command(&fixture, today - ChronoDuration::days(100));
            command.ends_on = Some(today - ChronoDuration::days(50));
            let created_recurrence = usecase.create_task_recurrence(command).await.unwrap();
            push_horizon_filled_to(
                &pool,
                created_recurrence.id.0,
                today - ChronoDuration::days(60),
            )
            .await;

            let created = usecase.ensure_recurrence_horizon_runs().await.unwrap();

            assert_eq!(created, 0);
            assert_eq!(run_count_for_org(&pool, fixture.organization_id).await, 0);

            let extended = usecase
                .extend_recurrence_horizons_for_organization(fixture.organization_id)
                .await
                .unwrap();
            assert_eq!(
                extended, 0,
                "a finished series is never materialized further, even called directly"
            );

            cleanup(&pool, fixture.organization_id, &[fixture.owner_id]).await;
        }
    }
}
