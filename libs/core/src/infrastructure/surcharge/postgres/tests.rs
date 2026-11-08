use common::{CoreError, generate_uuid_v7};
use sqlx::PgPool;

use crate::{
    OrganizationId, Surcharge, SurchargeId, SurchargeInput, SurchargeKind,
    application::test_support::dev_pool,
    domain::surcharge::ports::SurchargeRepository,
    infrastructure::{
        postgres::{SharedTx, with_tx},
        surcharge::postgres::PgSurchargeRepository,
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

fn surcharge(org: OrganizationId, name: &str, rate_bp: i32) -> Surcharge {
    Surcharge::new(
        SurchargeId(generate_uuid_v7()),
        org,
        SurchargeInput {
            name,
            rate_bp,
            kind: SurchargeKind::Night,
        },
        chrono::Utc::now(),
    )
    .unwrap()
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn a_surcharge_round_trips_and_is_scoped_by_organization() {
    run(async |tx| {
        let org = seed_organization(&tx).await;
        let other = seed_organization(&tx).await;
        let mut repository = PgSurchargeRepository::new(&tx);
        let stored = repository.insert(&surcharge(org, "Nuit", 2500)).await?;

        let found = repository.find(org, stored.id).await?;
        let foreign = repository.find(other, stored.id).await?;
        let mut archived = stored.clone();
        archived.archive(chrono::Utc::now());
        let updated = repository.update(&archived).await?;
        let listed = repository.list_by_organization(org).await?;

        assert_eq!(found, Some(stored));
        assert_eq!(foreign, None);
        assert!(updated.is_archived());
        assert_eq!(listed, vec![updated]);
        assert!(repository.list_by_organization(other).await?.is_empty());
        Ok(())
    })
    .await;
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn the_database_refuses_a_blank_name_an_unknown_kind_and_a_negative_rate() {
    run(async |tx| {
        let org = seed_organization(&tx).await;
        let mut refused = Vec::new();
        for (name, kind, rate) in [(" ", "NIGHT", 0), ("A", "WEEKEND", 0), ("A", "NIGHT", -1)] {
            let mut guard = tx.lock().await;
            sqlx::query("SAVEPOINT attempt")
                .execute(&mut ***guard)
                .await
                .unwrap();
            let outcome = sqlx::query(
                "INSERT INTO surcharges (id, org_id, name, rate_bp, kind) VALUES ($1, $2, $3, $4, $5)",
            )
            .bind(generate_uuid_v7())
            .bind(org.0)
            .bind(name)
            .bind(rate)
            .bind(kind)
            .execute(&mut ***guard)
            .await;
            refused.push(outcome.is_err());
            sqlx::query("ROLLBACK TO SAVEPOINT attempt")
                .execute(&mut ***guard)
                .await
                .unwrap();
        }

        assert_eq!(refused, vec![true, true, true]);
        Ok(())
    })
    .await;
}
