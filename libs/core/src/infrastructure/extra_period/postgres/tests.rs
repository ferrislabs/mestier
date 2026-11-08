use chrono::NaiveDate;
use common::{CoreError, generate_uuid_v7};
use sqlx::PgPool;

use crate::{
    ExtraPeriod, ExtraPeriodDraft, ExtraPeriodFilter, ExtraPeriodId, ExtraStatus, LunchInput,
    MemberId, OrganizationId,
    application::test_support::dev_pool,
    domain::extra_period::ports::ExtraPeriodRepository,
    infrastructure::{
        extra_period::postgres::PgExtraPeriodRepository,
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

fn period(org: OrganizationId, member: MemberId, lunch: Option<LunchInput>) -> ExtraPeriod {
    ExtraPeriod::declare(
        ExtraPeriodId(generate_uuid_v7()),
        org,
        &ExtraPeriodDraft {
            member_id: member,
            date: NaiveDate::from_ymd_opt(2026, 11, 14).unwrap(),
            start: "08:00".to_owned(),
            end: "18:00".to_owned(),
            lunch,
            reason: "Chantier".to_owned(),
            surcharge_id: None,
            rate_bp: Some(2500),
            approve: false,
        },
        None,
        None,
        chrono::Utc::now(),
    )
    .unwrap()
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn a_period_round_trips_with_its_lunch_and_decision() {
    run(async |tx| {
        let org = seed_organization(&tx).await;
        let member = seed_member(&tx, org).await;
        let mut repository = PgExtraPeriodRepository::new(&tx);
        let lunch = Some(LunchInput {
            start: "12:00".to_owned(),
            end: "13:00".to_owned(),
        });
        let stored = repository.insert(&period(org, member, lunch)).await?;

        let mut rejected = stored.clone();
        rejected
            .reject("Non", Some(member), chrono::Utc::now())
            .unwrap();
        let updated = repository.update(&rejected).await?;
        let found = repository.find(org, stored.id).await?;
        let (listed, total) = repository
            .list(
                org,
                &ExtraPeriodFilter {
                    status: Some(ExtraStatus::Rejected),
                    ..Default::default()
                },
                10,
                0,
            )
            .await?;

        assert_eq!(stored.hours.worked_minutes(), 540);
        assert_eq!(updated.status, ExtraStatus::Rejected);
        assert_eq!(updated.decision_comment.as_deref(), Some("Non"));
        assert_eq!(found, Some(updated.clone()));
        assert_eq!((listed, total), (vec![updated], 1));
        Ok(())
    })
    .await;
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn a_period_is_invisible_from_another_organization() {
    run(async |tx| {
        let org = seed_organization(&tx).await;
        let other = seed_organization(&tx).await;
        let member = seed_member(&tx, org).await;
        let mut repository = PgExtraPeriodRepository::new(&tx);
        let stored = repository.insert(&period(org, member, None)).await?;

        assert_eq!(repository.find(other, stored.id).await?, None);
        assert_eq!(
            repository
                .list(other, &ExtraPeriodFilter::default(), 10, 0)
                .await?,
            (vec![], 0)
        );
        Ok(())
    })
    .await;
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn the_database_refuses_an_inverted_span_and_a_rejection_without_comment() {
    run(async |tx| {
        let org = seed_organization(&tx).await;
        let member = seed_member(&tx, org).await;
        let mut refused = Vec::new();
        for (start, end, status, comment) in [
            ("09:00", "09:00", "PENDING", None),
            ("09:00", "08:00", "PENDING", None),
            ("08:00", "09:00", "REJECTED", None),
            ("08:00", "09:00", "REJECTED", Some("  ")),
        ] {
            let mut guard = tx.lock().await;
            sqlx::query("SAVEPOINT attempt")
                .execute(&mut ***guard)
                .await
                .unwrap();
            let outcome = sqlx::query(
                "INSERT INTO extra_periods (id, org_id, member_id, date, start_time, end_time, reason, rate_bp, status, decision_comment)
                 VALUES ($1, $2, $3, '2026-11-14', $4::time, $5::time, 'Chantier', 0, $6, $7)",
            )
            .bind(generate_uuid_v7())
            .bind(org.0)
            .bind(member.0)
            .bind(start)
            .bind(end)
            .bind(status)
            .bind(comment)
            .execute(&mut ***guard)
            .await;
            refused.push(outcome.is_err());
            sqlx::query("ROLLBACK TO SAVEPOINT attempt")
                .execute(&mut ***guard)
                .await
                .unwrap();
        }

        assert_eq!(refused, vec![true; 4]);
        Ok(())
    })
    .await;
}
