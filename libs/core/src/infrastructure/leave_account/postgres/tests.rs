use chrono::Utc;
use common::{CoreError, generate_uuid_v7};
use sqlx::PgPool;
use tokio::sync::OnceCell;
use uuid::Uuid;

use crate::{
    AcquisitionDraft, LeaveAccount, LeaveAccountId, LeaveAccountInput, OrganizationId,
    application::test_support::{dev_pool, scratch_pool},
    domain::leave_account::ports::LeaveAccountRepository,
    infrastructure::{
        leave_account::postgres::PgLeaveAccountRepository,
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

fn account(org: OrganizationId, name: &str, acquisition: AcquisitionDraft) -> LeaveAccount {
    LeaveAccount::new(
        LeaveAccountId(generate_uuid_v7()),
        org,
        LeaveAccountInput {
            name,
            acquisition,
            period_start_month: 6,
            carry_over_max: 5.0,
        },
        Utc::now(),
    )
    .unwrap()
}

async fn refuses(tx: &SharedTx<'_>, statement: &str, org: OrganizationId) -> bool {
    let mut guard = tx.lock().await;
    sqlx::query("SAVEPOINT attempt")
        .execute(&mut ***guard)
        .await
        .unwrap();
    let refused = sqlx::query(statement)
        .bind(generate_uuid_v7())
        .bind(org.0)
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
async fn spec_rh_4_1_decimals_round_trip_exactly_through_the_hundredths_columns() {
    run(async |tx| {
        let org = seed_organization(&tx).await;
        let mut repo = PgLeaveAccountRepository::new(&tx);
        for acquisition in [
            AcquisitionDraft::Monthly { per_month: 2.08 },
            AcquisitionDraft::Monthly { per_month: 0.1 },
            AcquisitionDraft::Annual { days: 10.0 },
        ] {
            let stored = repo.insert(&account(org, "Compte", acquisition)).await?;
            let read = repo.find(org, stored.id).await?.unwrap();
            assert_eq!(read, stored);
            let wire = read.acquisition.amount().days().as_decimal();
            match acquisition {
                AcquisitionDraft::Monthly {
                    per_month: expected,
                }
                | AcquisitionDraft::Annual { days: expected } => assert_eq!(wire, expected),
            }
        }
        assert_eq!(repo.list_by_organization(org).await?.len(), 3);
        Ok(())
    })
    .await;
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn spec_rh_4_1_the_database_refuses_an_out_of_range_account() {
    run(async |tx| {
        let org = seed_organization(&tx).await;
        for (statement, label) in [
            ("INSERT INTO leave_accounts (id, org_id, name, acquisition_kind, acquisition_hundredths, period_start_month, carry_over_max_hundredths) VALUES ($1, $2, 'A', 'ANNUAL', 100, 13, 0)", "month 13"),
            ("INSERT INTO leave_accounts (id, org_id, name, acquisition_kind, acquisition_hundredths, period_start_month, carry_over_max_hundredths) VALUES ($1, $2, 'A', 'ANNUAL', 0, 1, 0)", "zero acquisition"),
            ("INSERT INTO leave_accounts (id, org_id, name, acquisition_kind, acquisition_hundredths, period_start_month, carry_over_max_hundredths) VALUES ($1, $2, 'A', 'ANNUAL', 100, 1, -1)", "negative carry"),
            ("INSERT INTO leave_accounts (id, org_id, name, acquisition_kind, acquisition_hundredths, period_start_month, carry_over_max_hundredths) VALUES ($1, $2, ' ', 'ANNUAL', 100, 1, 0)", "blank name"),
            ("INSERT INTO leave_accounts (id, org_id, name, acquisition_kind, acquisition_hundredths, period_start_month, carry_over_max_hundredths) VALUES ($1, $2, 'A', 'WEEKLY', 100, 1, 0)", "unknown kind"),
        ] {
            assert!(refuses(&tx, statement, org).await, "{label}");
        }
        Ok(())
    })
    .await;
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn spec_rh_4_4_a_type_cannot_point_to_an_account_of_another_organization() {
    run(async |tx| {
        let org = seed_organization(&tx).await;
        let other = seed_organization(&tx).await;
        let mut repo = PgLeaveAccountRepository::new(&tx);
        let foreign = repo
            .insert(&account(other, "Etranger", AcquisitionDraft::Annual { days: 1.0 }))
            .await?;
        let mut guard = tx.lock().await;
        let refused = sqlx::query(
            "INSERT INTO absence_types (id, org_id, name, hue, count_mode, account_id, paid, needs_approval, needs_proof, kind)
             VALUES ($1, $2, 'T', 1, 'WORKING', $3, true, true, false, 'LEAVE')",
        )
        .bind(generate_uuid_v7())
        .bind(org.0)
        .bind(foreign.id.0)
        .execute(&mut ***guard)
        .await;
        assert!(refused.is_err());
        Ok(())
    })
    .await;
}

static MIGRATION_SCRATCH: OnceCell<String> = OnceCell::const_new();

const BEFORE_LEAVE_ACCOUNTS: i64 = 20261110000003;

async fn count(pool: &PgPool, statement: &str) -> i64 {
    sqlx::query_scalar(statement)
        .fetch_one(pool)
        .await
        .unwrap_or_else(|error| panic!("`{statement}` failed: {error}"))
}

async fn seed_organization_with_types(pool: &PgPool, with_types: bool) -> Uuid {
    let owner = Uuid::now_v7();
    let organization = Uuid::now_v7();
    sqlx::query("INSERT INTO users (id, email, username, display_name, sub) VALUES ($1, $2, $3, 'Owner', $4)")
        .bind(owner)
        .bind(format!("owner-{owner}@example.com"))
        .bind(format!("owner-{owner}"))
        .bind(format!("sub-{owner}"))
        .execute(pool)
        .await
        .unwrap();
    sqlx::query("INSERT INTO organizations (id, name, slug, owner_id) VALUES ($1, 'Org', $2, $3)")
        .bind(organization)
        .bind(format!("org-{organization}"))
        .bind(owner)
        .execute(pool)
        .await
        .unwrap();
    if with_types {
        for (name, account) in [
            ("Congés payés", Some(Uuid::now_v7())),
            ("RTT", None),
            ("Arrêt maladie", None),
        ] {
            sqlx::query(
                "INSERT INTO absence_types (id, org_id, name, hue, count_mode, account_id, paid, needs_approval, needs_proof, kind)
                 VALUES ($1, $2, $3, 1, 'WORKING', $4, true, true, false, 'LEAVE')",
            )
            .bind(Uuid::now_v7())
            .bind(organization)
            .bind(name)
            .bind(account)
            .execute(pool)
            .await
            .unwrap();
        }
    }
    organization
}

async fn assert_linked(pool: &PgPool, organization: Uuid) {
    let linked: Vec<(String, String)> = sqlx::query_as(
        "SELECT t.name, a.name FROM absence_types t JOIN leave_accounts a ON a.id = t.account_id AND a.org_id = t.org_id
         WHERE t.org_id = $1 ORDER BY t.name",
    )
    .bind(organization)
    .fetch_all(pool)
    .await
    .unwrap();
    assert_eq!(
        linked,
        [
            ("Congés payés".to_owned(), "Congés payés".to_owned()),
            ("RTT".to_owned(), "RTT".to_owned()),
        ]
    );
    assert_eq!(
        count(pool, &format!("SELECT COUNT(*) FROM absence_types WHERE org_id = '{organization}' AND account_id IS NOT NULL")).await,
        2
    );
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn spec_rh_4_1_the_migration_links_the_default_types_to_their_accounts_and_reverts() {
    let pool = scratch_pool("mestier_leave_migration_test_", &MIGRATION_SCRATCH).await;
    let migrator = sqlx::migrate!("../../migrations");

    migrator.undo(&pool, BEFORE_LEAVE_ACCOUNTS).await.unwrap();
    let with_types = seed_organization_with_types(&pool, true).await;
    let without_types = seed_organization_with_types(&pool, false).await;

    migrator.run(&pool).await.unwrap();
    assert_linked(&pool, with_types).await;
    assert_eq!(
        count(
            &pool,
            &format!("SELECT COUNT(*) FROM leave_accounts WHERE org_id = '{with_types}'")
        )
        .await,
        2
    );
    assert_eq!(
        count(
            &pool,
            &format!("SELECT COUNT(*) FROM leave_accounts WHERE org_id = '{without_types}'")
        )
        .await,
        0
    );

    migrator.undo(&pool, BEFORE_LEAVE_ACCOUNTS).await.unwrap();
    assert_eq!(
        count(&pool, "SELECT COUNT(*) FROM information_schema.tables WHERE table_name IN ('leave_accounts', 'leave_adjustments')").await,
        0
    );
    assert_eq!(
        count(
            &pool,
            "SELECT COUNT(*) FROM absence_types WHERE account_id IS NOT NULL"
        )
        .await,
        0
    );

    migrator.run(&pool).await.unwrap();
    assert_linked(&pool, with_types).await;
}
