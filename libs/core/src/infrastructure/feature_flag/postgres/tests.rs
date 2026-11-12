use common::{CoreError, OrganizationId, UserId, generate_uuid_v7};
use sqlx::PgPool;
use tokio::sync::OnceCell;

use crate::{
    FeatureFlag,
    application::test_support::{dev_pool, scratch_pool},
    domain::feature_flag::ports::FeatureFlagRepository,
    infrastructure::{
        feature_flag::postgres::PgFeatureFlagRepository,
        postgres::{SharedTx, with_tx},
    },
};

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

async fn seed(tx: &SharedTx<'_>) -> (UserId, OrganizationId, OrganizationId) {
    let user = generate_uuid_v7();
    let mut guard = tx.lock().await;
    sqlx::query(
        "INSERT INTO users (id, email, username, display_name, sub) VALUES ($1, $2, $3, 'Staff', $4)",
    )
    .bind(user)
    .bind(format!("staff-{user}@example.com"))
    .bind(format!("staff-{user}"))
    .bind(format!("sub-{user}"))
    .execute(&mut ***guard)
    .await
    .unwrap();
    let mut organizations = Vec::new();
    for _ in 0..2 {
        let id = generate_uuid_v7();
        sqlx::query(
            "INSERT INTO organizations (id, name, slug, owner_id) VALUES ($1, 'Org', $2, $3)",
        )
        .bind(id)
        .bind(format!("org-{id}"))
        .bind(user)
        .execute(&mut ***guard)
        .await
        .unwrap();
        organizations.push(OrganizationId(id));
    }
    (UserId(user), organizations[0], organizations[1])
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn a_flag_is_disabled_until_a_row_enables_it_and_only_for_that_organization() {
    run(async |tx| {
        let (staff, a, b) = seed(&tx).await;
        let mut repository = PgFeatureFlagRepository::new(&tx);

        let before = repository.is_enabled(a, FeatureFlag::DriveAi).await?;
        repository.enable(a, FeatureFlag::DriveAi, staff).await?;

        assert!(!before);
        assert!(repository.is_enabled(a, FeatureFlag::DriveAi).await?);
        assert!(!repository.is_enabled(b, FeatureFlag::DriveAi).await?);
        assert!(repository.list_enabled(b).await?.is_empty());
        Ok(())
    })
    .await;
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn enable_and_disable_report_whether_anything_changed() {
    run(async |tx| {
        let (staff, a, _) = seed(&tx).await;
        let mut repository = PgFeatureFlagRepository::new(&tx);

        let first_enable = repository.enable(a, FeatureFlag::DriveAi, staff).await?;
        let second_enable = repository.enable(a, FeatureFlag::DriveAi, staff).await?;
        let first_disable = repository.disable(a, FeatureFlag::DriveAi).await?;
        let second_disable = repository.disable(a, FeatureFlag::DriveAi).await?;

        assert!(first_enable);
        assert!(!second_enable);
        assert!(first_disable);
        assert!(!second_disable);
        Ok(())
    })
    .await;
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn an_activation_remembers_who_enabled_it_and_unknown_keys_are_ignored() {
    run(async |tx| {
        let (staff, a, _) = seed(&tx).await;
        sqlx::query(
            "INSERT INTO organization_feature_flags (org_id, flag) VALUES ($1, 'retired_flag')",
        )
        .bind(a.0)
        .execute(&mut ***tx.lock().await)
        .await
        .unwrap();
        let mut repository = PgFeatureFlagRepository::new(&tx);
        repository.enable(a, FeatureFlag::DriveAi, staff).await?;

        let activations = repository.list_enabled(a).await?;

        assert_eq!(activations.len(), 1);
        assert_eq!(activations[0].flag, FeatureFlag::DriveAi);
        assert_eq!(activations[0].enabled_by, Some(staff));
        Ok(())
    })
    .await;
}

static MIGRATION_SCRATCH: OnceCell<String> = OnceCell::const_new();

async fn flags_table_exists(pool: &PgPool) -> bool {
    sqlx::query_scalar::<_, bool>(
        "SELECT to_regclass('public.organization_feature_flags') IS NOT NULL",
    )
    .fetch_one(pool)
    .await
    .unwrap()
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn the_feature_flags_migration_goes_up_down_and_up_again() {
    let pool = scratch_pool("mestier_ffmig_test_", &MIGRATION_SCRATCH).await;
    let migrator = sqlx::migrate!("../../migrations");

    let applied = flags_table_exists(&pool).await;
    migrator.undo(&pool, 20261120000001).await.unwrap();
    let reverted = flags_table_exists(&pool).await;
    migrator.run(&pool).await.unwrap();
    let reapplied = flags_table_exists(&pool).await;

    assert!(applied);
    assert!(!reverted);
    assert!(reapplied);
}
