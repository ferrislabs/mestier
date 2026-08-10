use std::time::Duration;

use common::{CoreError, OrganizationId};
use mestier_macros::repository;

use crate::{
    domain::automation::{ports::AutomationSettingsRepository, settings::AutomationSettings},
    infrastructure::postgres::{SharedTx, error::map_sqlx_error},
};

/// Reads `automation.settings`. Formerly part of the delivery pipeline #201
/// replaces (a webhook retry needed the same organization-wide backoff
/// schedule); moved here, under its own name, once delivery was gone and
/// `DeliveryRepository` had nothing left to do with delivery.
///
/// `application::automation::run::retry_schedule_for` is why this still
/// exists as a repository rather than a free function: it reuses this port
/// verbatim for the run engine's own retry backoff.
#[repository(domain = AutomationSettings, backend = Postgres)]
pub struct PgAutomationSettingsRepository<'tx> {
    tx: SharedTx<'tx>,
}

impl<'tx> PgAutomationSettingsRepository<'tx> {
    pub fn new(tx: &SharedTx<'tx>) -> Self {
        Self { tx: tx.clone() }
    }
}

impl<'tx> AutomationSettingsRepository for PgAutomationSettingsRepository<'tx> {
    async fn settings_for(
        &mut self,
        org_id: OrganizationId,
    ) -> Result<AutomationSettings, CoreError> {
        let mut tx = self.tx.lock().await;

        let row = sqlx::query!(
            r#"
            SELECT event_retention_seconds,
                   succeeded_delivery_retention_seconds,
                   retry_schedule_seconds,
                   disable_target_after
            FROM automation.settings
            WHERE org_id = $1
            "#,
            org_id.0,
        )
        .fetch_optional(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        // No row means the organization never changed anything. Writing one on
        // first read would turn every read into a write.
        let Some(row) = row else {
            return Ok(AutomationSettings::default());
        };

        Ok(AutomationSettings {
            event_retention: Duration::from_secs(row.event_retention_seconds.max(0) as u64),
            succeeded_delivery_retention: Duration::from_secs(
                row.succeeded_delivery_retention_seconds.max(0) as u64,
            ),
            retry_schedule: row
                .retry_schedule_seconds
                .into_iter()
                .map(|seconds| Duration::from_secs(seconds.max(0) as u64))
                .collect(),
            disable_target_after: row
                .disable_target_after
                .map(|threshold| threshold.max(0) as u32),
        })
    }
}

#[cfg(test)]
mod tests {
    use common::generate_uuid_v7;
    use sqlx::PgPool;

    use super::*;
    use crate::infrastructure::postgres::with_tx;

    async fn make_pool() -> PgPool {
        let url = std::env::var("DATABASE_URL")
            .expect("DATABASE_URL must be set to run settings integration tests");
        PgPool::connect(&url).await.unwrap()
    }

    async fn seed_organization(pool: &PgPool) -> OrganizationId {
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

        OrganizationId(org_id)
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn an_organization_without_a_settings_row_gets_the_defaults() {
        let pool = make_pool().await;
        let org_id = seed_organization(&pool).await;

        let settings = with_tx(&pool, async |tx| {
            let mut repo = PgAutomationSettingsRepository::new(&tx);
            repo.settings_for(org_id).await
        })
        .await
        .unwrap();

        assert_eq!(settings, AutomationSettings::default());
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn an_organizations_configured_retry_schedule_is_read_back() {
        let pool = make_pool().await;
        let org_id = seed_organization(&pool).await;
        sqlx::query!(
            r#"INSERT INTO automation.settings (org_id, retry_schedule_seconds)
               VALUES ($1, ARRAY[1, 2, 3]::bigint[])"#,
            org_id.0,
        )
        .execute(&pool)
        .await
        .unwrap();

        let settings = with_tx(&pool, async |tx| {
            let mut repo = PgAutomationSettingsRepository::new(&tx);
            repo.settings_for(org_id).await
        })
        .await
        .unwrap();

        assert_eq!(
            settings.retry_schedule,
            vec![
                Duration::from_secs(1),
                Duration::from_secs(2),
                Duration::from_secs(3),
            ]
        );
    }
}
