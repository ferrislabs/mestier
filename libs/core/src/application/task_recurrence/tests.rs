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

        let created = usecase
            .create_task_recurrence(daily_command(
                &fixture,
                NaiveDate::from_ymd_opt(2026, 8, 24).unwrap(),
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
}
