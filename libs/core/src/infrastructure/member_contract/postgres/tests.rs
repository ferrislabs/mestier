use chrono::{NaiveDate, Utc};
use common::{CoreError, generate_uuid_v7};
use serde_json::json;
use sqlx::PgPool;

use crate::{
    ContractSchedule, ContractType, MemberContract, MemberContractId, MemberContractInput,
    MemberId, OrganizationId, ScheduleInput,
    application::test_support::dev_pool,
    domain::member_contract::ports::MemberContractRepository,
    infrastructure::{
        member_contract::postgres::PgMemberContractRepository,
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

fn date(year: i32, month: u32, day: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(year, month, day).unwrap()
}

fn schedule(value: serde_json::Value) -> ScheduleInput {
    serde_json::from_value(value).unwrap()
}

fn week() -> serde_json::Value {
    let open =
        json!({ "start": "08:00", "end": "17:00", "lunch": { "start": "12:00", "end": "13:00" } });
    json!([open, open, open, open, open, null, null])
}

fn contract(
    org: OrganizationId,
    member: MemberId,
    start: NaiveDate,
    end: Option<NaiveDate>,
    wire: serde_json::Value,
) -> MemberContract {
    MemberContract::draft(
        MemberContractId(generate_uuid_v7()),
        org,
        member,
        MemberContractInput {
            contract_type: ContractType::Cdi,
            start_date: start,
            end_date: end,
            trial_end_date: None,
            schedule: schedule(wire),
        },
        Utc::now(),
    )
    .unwrap()
}

async fn refused_by_the_database(
    tx: &SharedTx<'_>,
    repo: &mut PgMemberContractRepository<'_>,
    contract: &MemberContract,
) -> bool {
    {
        let mut guard = tx.lock().await;
        sqlx::query("SAVEPOINT attempt")
            .execute(&mut ***guard)
            .await
            .unwrap();
    }
    let refused = repo.insert(contract).await.is_err();
    {
        let mut guard = tx.lock().await;
        sqlx::query("ROLLBACK TO SAVEPOINT attempt")
            .execute(&mut ***guard)
            .await
            .unwrap();
    }
    refused
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn spec_rh_2_1_the_three_schedule_kinds_round_trip_through_the_jsonb_column() {
    run(async |tx| {
        let org = seed_organization(&tx).await;
        let member = seed_member(&tx, org).await;
        let mut repo = PgMemberContractRepository::new(&tx);
        let wires = [
            json!({ "kind": "weekly", "week": week() }),
            json!({ "kind": "rotation", "anchor": "2026-01-05", "blocks": [
                { "length": 2, "unit": "weeks", "label": "Chantier", "week": week() },
                { "length": 1, "unit": "months", "label": "Repos", "week": [null, null, null, null, null, null, null] },
            ] }),
            json!({ "kind": "variable", "default_day": { "start": "09:00", "end": "17:30" } }),
        ];
        for (index, wire) in wires.into_iter().enumerate() {
            let start = date(2020 + i32::try_from(index).unwrap(), 1, 1);
            let end = date(2020 + i32::try_from(index).unwrap(), 12, 31);
            let stored = repo.insert(&contract(org, member, start, Some(end), wire.clone())).await?;
            let read = repo.find(org, stored.id).await?.unwrap();
            assert_eq!(read, stored);
            assert_eq!(serde_json::to_value(ScheduleInput::from(&read.schedule)).unwrap(), wire);
        }
        assert_eq!(repo.list_by_member(org, member).await?.len(), 3);
        Ok(())
    })
    .await;
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn spec_rh_2_1_a_stored_schedule_that_breaks_the_invariants_is_refused_on_read() {
    run(async |tx| {
        let org = seed_organization(&tx).await;
        let member = seed_member(&tx, org).await;
        let mut repo = PgMemberContractRepository::new(&tx);
        let stored = repo
            .insert(&contract(
                org,
                member,
                date(2026, 1, 1),
                None,
                json!({ "kind": "weekly", "week": week() }),
            ))
            .await?;
        {
            let mut guard = tx.lock().await;
            sqlx::query("UPDATE member_contracts SET schedule = $1 WHERE id = $2")
                .bind(json!({ "kind": "weekly", "week": [null] }))
                .bind(stored.id.0)
                .execute(&mut ***guard)
                .await
                .unwrap();
        }
        assert!(matches!(
            repo.find(org, stored.id).await,
            Err(CoreError::Internal(_))
        ));
        Ok(())
    })
    .await;
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn spec_rh_2_3_the_database_refuses_two_contracts_sharing_a_day() {
    run(async |tx| {
        let org = seed_organization(&tx).await;
        let member = seed_member(&tx, org).await;
        let other = seed_member(&tx, org).await;
        let mut repo = PgMemberContractRepository::new(&tx);
        let wire = json!({ "kind": "weekly", "week": week() });
        repo.insert(&contract(
            org,
            member,
            date(2026, 1, 1),
            Some(date(2026, 6, 30)),
            wire.clone(),
        ))
        .await?;

        let sharing_a_day = contract(org, member, date(2026, 6, 30), None, wire.clone());
        assert!(refused_by_the_database(&tx, &mut repo, &sharing_a_day).await);
        let inside = contract(
            org,
            member,
            date(2026, 2, 1),
            Some(date(2026, 3, 1)),
            wire.clone(),
        );
        assert!(refused_by_the_database(&tx, &mut repo, &inside).await);

        repo.insert(&contract(org, member, date(2026, 7, 1), None, wire.clone()))
            .await?;
        let after_open_end = contract(org, member, date(2030, 1, 1), None, wire.clone());
        assert!(refused_by_the_database(&tx, &mut repo, &after_open_end).await);
        repo.insert(&contract(org, other, date(2026, 1, 1), None, wire))
            .await?;
        Ok(())
    })
    .await;
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn spec_rh_2_4_the_database_refuses_a_trial_end_outside_the_contract() {
    run(async |tx| {
        let org = seed_organization(&tx).await;
        let member = seed_member(&tx, org).await;
        let mut repo = PgMemberContractRepository::new(&tx);
        let mut inside = contract(org, member, date(2026, 3, 1), Some(date(2026, 9, 30)), json!({ "kind": "weekly", "week": week() }));
        {
            let mut guard = tx.lock().await;
            sqlx::query("SAVEPOINT attempt").execute(&mut ***guard).await.unwrap();
            let outside = sqlx::query(
                "INSERT INTO member_contracts (id, org_id, member_id, type, start_date, end_date, trial_end_date, schedule)
                 VALUES ($1, $2, $3, 'CDI', '2026-03-01', '2026-09-30', '2026-10-01', '{}')",
            )
            .bind(generate_uuid_v7())
            .bind(org.0)
            .bind(member.0)
            .execute(&mut ***guard)
            .await;
            assert!(outside.is_err());
            sqlx::query("ROLLBACK TO SAVEPOINT attempt").execute(&mut ***guard).await.unwrap();
        }
        inside = repo.insert(&inside).await?;
        assert_eq!(inside.period.trial_end(), None);
        Ok(())
    })
    .await;
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn contracts_are_updated_deleted_and_scoped_to_their_organization() {
    run(async |tx| {
        let org = seed_organization(&tx).await;
        let other = seed_organization(&tx).await;
        let member = seed_member(&tx, org).await;
        let mut repo = PgMemberContractRepository::new(&tx);
        let wire = json!({ "kind": "weekly", "week": week() });
        let stored = repo.insert(&contract(org, member, date(2026, 1, 1), None, wire)).await?;

        assert_eq!(repo.find(other, stored.id).await?, None);
        assert!(repo.list_by_organization(other).await?.is_empty());
        assert_eq!(repo.list_by_organization(org).await?.len(), 1);

        let moved = stored
            .replaced_with(
                MemberContractInput {
                    contract_type: ContractType::Manager,
                    start_date: date(2026, 2, 1),
                    end_date: None,
                    trial_end_date: Some(date(2026, 3, 1)),
                    schedule: schedule(json!({ "kind": "variable", "default_day": { "start": "09:00", "end": "17:00" } })),
                },
                Utc::now(),
            )
            .unwrap();
        let updated = repo.update(&moved).await?;
        assert_eq!(updated.contract_type, ContractType::Manager);
        assert!(matches!(updated.schedule, ContractSchedule::Variable { .. }));

        repo.delete(other, stored.id).await?;
        assert!(repo.find(org, stored.id).await?.is_some());
        repo.delete(org, stored.id).await?;
        assert_eq!(repo.find(org, stored.id).await?, None);
        Ok(())
    })
    .await;
}
