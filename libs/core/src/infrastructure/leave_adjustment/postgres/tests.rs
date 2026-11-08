use common::{CoreError, generate_uuid_v7};
use sqlx::PgPool;

use crate::{
    AcquisitionDraft, LeaveAccount, LeaveAccountId, LeaveAccountInput, MemberId, OrganizationId,
    application::test_support::dev_pool,
    domain::{
        leave_account::ports::LeaveAccountRepository,
        leave_adjustment::ports::LeaveAdjustmentRepository,
    },
    infrastructure::{
        leave_account::postgres::PgLeaveAccountRepository,
        leave_adjustment::postgres::PgLeaveAdjustmentRepository,
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

async fn seed_account(tx: &SharedTx<'_>, org: OrganizationId) -> LeaveAccount {
    let account = LeaveAccount::new(
        LeaveAccountId(generate_uuid_v7()),
        org,
        LeaveAccountInput {
            name: "CP",
            acquisition: AcquisitionDraft::Annual { days: 5.0 },
            period_start_month: 1,
            carry_over_max: 0.0,
        },
        chrono::Utc::now(),
    )
    .unwrap();
    PgLeaveAccountRepository::new(tx)
        .insert(&account)
        .await
        .unwrap()
}

async fn insert_raw(
    tx: &SharedTx<'_>,
    org: OrganizationId,
    member: MemberId,
    account: LeaveAccountId,
    days: i32,
    reason: &str,
) -> bool {
    let mut guard = tx.lock().await;
    sqlx::query("SAVEPOINT attempt")
        .execute(&mut ***guard)
        .await
        .unwrap();
    let refused = sqlx::query(
        "INSERT INTO leave_adjustments (id, org_id, member_id, account_id, date, days_hundredths, kind, reason)
         VALUES ($1, $2, $3, $4, '2026-07-01', $5, 'ADJUSTMENT', $6)",
    )
    .bind(generate_uuid_v7())
    .bind(org.0)
    .bind(member.0)
    .bind(account.0)
    .bind(days)
    .bind(reason)
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
async fn spec_rh_4_2_the_database_refuses_zero_days_a_blank_reason_and_foreign_rows() {
    run(async |tx| {
        let org = seed_organization(&tx).await;
        let other = seed_organization(&tx).await;
        let member = seed_member(&tx, org).await;
        let other_member = seed_member(&tx, other).await;
        let account = seed_account(&tx, org).await;
        let other_account = seed_account(&tx, other).await;

        assert!(!insert_raw(&tx, org, member, account.id, -150, "Erreur").await);
        assert!(insert_raw(&tx, org, member, account.id, 0, "Erreur").await);
        assert!(insert_raw(&tx, org, member, account.id, 100, "  ").await);
        assert!(insert_raw(&tx, org, other_member, account.id, 100, "x").await);
        assert!(insert_raw(&tx, org, member, other_account.id, 100, "x").await);
        Ok(())
    })
    .await;
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn spec_rh_4_3_a_deleted_adjustment_leaves_the_list() {
    run(async |tx| {
        let org = seed_organization(&tx).await;
        let member = seed_member(&tx, org).await;
        let account = seed_account(&tx, org).await;
        let mut repo = PgLeaveAdjustmentRepository::new(&tx);
        let draft = crate::LeaveAdjustmentDraft {
            member_id: member,
            account_id: account.id,
            date: chrono::NaiveDate::from_ymd_opt(2026, 7, 1).unwrap(),
            days: -0.25,
            kind: crate::AdjustmentKind::Adjustment,
            reason: "Geste".to_owned(),
        };
        let adjustment = crate::LeaveAdjustment::record(
            crate::LeaveAdjustmentId(generate_uuid_v7()),
            org,
            &draft,
            &account,
            Some(member),
            chrono::Utc::now(),
        )
        .unwrap();

        let stored = repo.insert(&adjustment).await?;
        assert_eq!(stored.days.days().as_decimal(), -0.25);
        assert_eq!(stored.author_member_id, Some(member));
        assert_eq!(
            repo.list(org, Some(member), Some(account.id)).await?.len(),
            1
        );
        repo.delete(org, stored.id).await?;
        assert!(repo.list(org, None, None).await?.is_empty());
        assert!(repo.find(org, stored.id).await?.is_none());
        Ok(())
    })
    .await;
}
