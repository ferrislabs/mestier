use chrono::{NaiveDate, Utc};
use common::{CoreError, generate_uuid_v7};
use sqlx::PgPool;

use crate::{
    DayOverride, DayOverrideId, DayOverrideInput, LunchInput, MemberId, OrganizationId,
    OverrideKind, PlanningPeriod,
    application::test_support::dev_pool,
    domain::day_override::ports::DayOverrideRepository,
    infrastructure::{
        day_override::postgres::PgDayOverrideRepository,
        postgres::{SharedTx, with_tx},
    },
};

async fn seed_organization(tx: &SharedTx<'_>) -> OrganizationId {
    let mut guard = tx.lock().await;
    let user_id = generate_uuid_v7();
    sqlx::query!(
        r#"INSERT INTO users (id, email, username, display_name, sub)
           VALUES ($1, $2, $3, $4, $5)"#,
        user_id,
        format!("owner-{user_id}@example.com"),
        format!("owner-{user_id}"),
        "Owner",
        format!("sub-{user_id}"),
    )
    .execute(&mut ***guard)
    .await
    .unwrap();

    let org_id = generate_uuid_v7();
    sqlx::query!(
        r#"INSERT INTO organizations (id, name, slug, owner_id) VALUES ($1, $2, $3, $4)"#,
        org_id,
        "Test Org",
        format!("test-org-{org_id}"),
        user_id,
    )
    .execute(&mut ***guard)
    .await
    .unwrap();

    OrganizationId(org_id)
}

async fn seed_member(tx: &SharedTx<'_>, organization_id: OrganizationId) -> MemberId {
    let mut guard = tx.lock().await;
    let id = generate_uuid_v7();
    sqlx::query!(
        r#"INSERT INTO organization_members (id, organization_id, last_name) VALUES ($1, $2, $3)"#,
        id,
        organization_id.0,
        "Claire",
    )
    .execute(&mut ***guard)
    .await
    .unwrap();

    MemberId(id)
}

async fn run<F>(work: F)
where
    F: AsyncFnOnce(SharedTx<'_>) -> Result<(), CoreError>,
{
    let pool: PgPool = dev_pool().await;
    let outcome = with_tx(&pool, async |tx| {
        work(tx).await?;
        Err::<(), _>(CoreError::Internal("rollback".into()))
    })
    .await;
    assert!(matches!(outcome, Err(CoreError::Internal(message)) if message == "rollback"));
}

fn date(day: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(2026, 11, day).unwrap()
}

fn declare(
    org: OrganizationId,
    member: MemberId,
    day: u32,
    input: &DayOverrideInput,
) -> DayOverride {
    DayOverride::declare(
        DayOverrideId(generate_uuid_v7()),
        org,
        member,
        date(day),
        input,
        Utc::now(),
    )
    .unwrap()
}

fn work(start: &str, end: &str, note: &str) -> DayOverrideInput {
    DayOverrideInput {
        kind: OverrideKind::Work,
        start: Some(start.to_owned()),
        end: Some(end.to_owned()),
        lunch: Some(LunchInput {
            start: "12:00".to_owned(),
            end: "13:00".to_owned(),
        }),
        note: note.to_owned(),
    }
}

fn off() -> DayOverrideInput {
    DayOverrideInput {
        kind: OverrideKind::Off,
        start: None,
        end: None,
        lunch: None,
        note: "Pont".to_owned(),
    }
}

fn period(from: u32, to: u32) -> PlanningPeriod {
    PlanningPeriod::new(date(from), date(to), 62).unwrap()
}

async fn refused_raw(
    tx: &SharedTx<'_>,
    org: OrganizationId,
    member: MemberId,
    sql_values: &str,
) -> bool {
    let mut guard = tx.lock().await;
    sqlx::query("SAVEPOINT attempt")
        .execute(&mut ***guard)
        .await
        .unwrap();
    let statement = format!(
        "INSERT INTO day_overrides (id, org_id, member_id, date, kind, start_time, end_time, lunch_start, lunch_end) VALUES ($1, $2, $3, '2026-11-20', {sql_values})"
    );
    let refused = sqlx::query(&statement)
        .bind(generate_uuid_v7())
        .bind(org.0)
        .bind(member.0)
        .execute(&mut ***guard)
        .await
        .is_err();
    sqlx::query("ROLLBACK TO SAVEPOINT attempt")
        .execute(&mut ***guard)
        .await
        .unwrap();
    refused
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn spec_rh_5_1_a_second_upsert_replaces_the_first_and_keeps_one_row() {
    run(async |tx| {
        let org = seed_organization(&tx).await;
        let member = seed_member(&tx, org).await;
        let mut repository = PgDayOverrideRepository::new(&tx);

        let first = repository
            .upsert(&declare(
                org,
                member,
                10,
                &work("08:00", "17:00", "Chantier"),
            ))
            .await?;
        let second = repository.upsert(&declare(org, member, 10, &off())).await?;

        let listed = repository.list(org, period(1, 30), None).await?;
        assert_eq!(listed, vec![second.clone()]);
        assert_eq!(second.id, first.id);
        assert_eq!(second.shape.kind(), OverrideKind::Off);
        assert_eq!(second.note, "Pont");
        Ok(())
    })
    .await;
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn spec_rh_5_3_the_list_keeps_only_the_period_the_member_and_the_organization() {
    run(async |tx| {
        let org = seed_organization(&tx).await;
        let other_org = seed_organization(&tx).await;
        let member = seed_member(&tx, org).await;
        let colleague = seed_member(&tx, org).await;
        let stranger = seed_member(&tx, other_org).await;
        let mut repository = PgDayOverrideRepository::new(&tx);
        for (owner, who, day) in [
            (org, member, 8),
            (org, member, 9),
            (org, member, 15),
            (org, member, 16),
            (org, colleague, 10),
            (other_org, stranger, 10),
        ] {
            repository.upsert(&declare(owner, who, day, &off())).await?;
        }

        let days: Vec<_> = repository
            .list(org, period(9, 15), None)
            .await?
            .into_iter()
            .map(|row| (row.member_id, row.date))
            .collect();
        let mine = repository.list(org, period(9, 15), Some(member)).await?;

        assert_eq!(
            days,
            vec![(member, date(9)), (colleague, date(10)), (member, date(15))]
        );
        assert_eq!(mine.len(), 2);
        Ok(())
    })
    .await;
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn spec_rh_5_2_the_database_refuses_inverted_hours_and_hours_on_a_day_off() {
    run(async |tx| {
        let org = seed_organization(&tx).await;
        let other = seed_organization(&tx).await;
        let member = seed_member(&tx, org).await;

        assert!(refused_raw(&tx, org, member, "'WORK', '17:00', '08:00', NULL, NULL").await);
        assert!(refused_raw(&tx, org, member, "'WORK', NULL, NULL, NULL, NULL").await);
        assert!(refused_raw(&tx, org, member, "'WORK', '08:00', '17:00', '12:00', NULL").await);
        assert!(
            refused_raw(
                &tx,
                org,
                member,
                "'WORK', '08:00', '17:00', '07:00', '13:00'"
            )
            .await
        );
        assert!(refused_raw(&tx, org, member, "'OFF', '08:00', '17:00', NULL, NULL").await);
        assert!(refused_raw(&tx, other, member, "'OFF', NULL, NULL, NULL, NULL").await);
        assert!(
            !refused_raw(
                &tx,
                org,
                member,
                "'WORK', '08:00', '17:00', '12:00', '13:00'"
            )
            .await
        );
        Ok(())
    })
    .await;
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn deleting_reports_whether_a_row_was_removed() {
    run(async |tx| {
        let org = seed_organization(&tx).await;
        let member = seed_member(&tx, org).await;
        let mut repository = PgDayOverrideRepository::new(&tx);
        repository.upsert(&declare(org, member, 10, &off())).await?;

        assert!(repository.delete(org, member, date(10)).await?);
        assert!(!repository.delete(org, member, date(10)).await?);
        assert!(repository.list(org, period(1, 30), None).await?.is_empty());
        Ok(())
    })
    .await;
}
