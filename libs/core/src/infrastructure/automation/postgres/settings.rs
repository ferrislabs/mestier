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

/// Shared by every query in this file that reads the four retention/backoff
/// columns back — `settings_for`, `upsert`'s `RETURNING`, and `all_settings`
/// each hit a differently-shaped anonymous row type (a fresh one per
/// `sqlx::query!` call site), so this takes the columns already unpacked
/// rather than one of those types.
fn settings_from_columns(
    event_retention_seconds: i64,
    succeeded_run_retention_seconds: i64,
    retry_schedule_seconds: Vec<i64>,
    disable_target_after: Option<i32>,
) -> AutomationSettings {
    AutomationSettings {
        event_retention: Duration::from_secs(event_retention_seconds.max(0) as u64),
        succeeded_run_retention: Duration::from_secs(succeeded_run_retention_seconds.max(0) as u64),
        retry_schedule: retry_schedule_seconds
            .into_iter()
            .map(|seconds| Duration::from_secs(seconds.max(0) as u64))
            .collect(),
        disable_target_after: disable_target_after.map(|threshold| threshold.max(0) as u32),
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
                   succeeded_run_retention_seconds,
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

        Ok(settings_from_columns(
            row.event_retention_seconds,
            row.succeeded_run_retention_seconds,
            row.retry_schedule_seconds,
            row.disable_target_after,
        ))
    }

    async fn upsert(
        &mut self,
        org_id: OrganizationId,
        settings: &AutomationSettings,
    ) -> Result<AutomationSettings, CoreError> {
        let mut tx = self.tx.lock().await;

        let retry_schedule_seconds: Vec<i64> = settings
            .retry_schedule
            .iter()
            .map(|interval| interval.as_secs() as i64)
            .collect();
        let disable_target_after = settings.disable_target_after.map(|v| v as i32);

        let row = sqlx::query!(
            r#"
            INSERT INTO automation.settings
                (org_id, event_retention_seconds, succeeded_run_retention_seconds,
                 retry_schedule_seconds, disable_target_after)
            VALUES ($1, $2, $3, $4, $5)
            ON CONFLICT (org_id) DO UPDATE
            SET event_retention_seconds = EXCLUDED.event_retention_seconds,
                succeeded_run_retention_seconds = EXCLUDED.succeeded_run_retention_seconds,
                retry_schedule_seconds = EXCLUDED.retry_schedule_seconds,
                disable_target_after = EXCLUDED.disable_target_after,
                updated_at = now()
            RETURNING event_retention_seconds, succeeded_run_retention_seconds,
                      retry_schedule_seconds, disable_target_after
            "#,
            org_id.0,
            settings.event_retention.as_secs() as i64,
            settings.succeeded_run_retention.as_secs() as i64,
            &retry_schedule_seconds,
            disable_target_after,
        )
        .fetch_one(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        Ok(settings_from_columns(
            row.event_retention_seconds,
            row.succeeded_run_retention_seconds,
            row.retry_schedule_seconds,
            row.disable_target_after,
        ))
    }

    async fn all_settings(
        &mut self,
    ) -> Result<Vec<(OrganizationId, AutomationSettings)>, CoreError> {
        let mut tx = self.tx.lock().await;

        let rows = sqlx::query!(
            r#"
            SELECT org_id,
                   event_retention_seconds,
                   succeeded_run_retention_seconds,
                   retry_schedule_seconds,
                   disable_target_after
            FROM automation.settings
            "#,
        )
        .fetch_all(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        Ok(rows
            .into_iter()
            .map(|row| {
                (
                    OrganizationId(row.org_id),
                    settings_from_columns(
                        row.event_retention_seconds,
                        row.succeeded_run_retention_seconds,
                        row.retry_schedule_seconds,
                        row.disable_target_after,
                    ),
                )
            })
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use common::generate_uuid_v7;
    use sqlx::PgPool;

    use super::*;
    use crate::application::test_support::automation_pool;
    use crate::infrastructure::postgres::with_tx;

    async fn make_pool() -> PgPool {
        automation_pool().await
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

    fn custom_settings() -> AutomationSettings {
        AutomationSettings {
            event_retention: Duration::from_secs(10 * 24 * 3600),
            succeeded_run_retention: Duration::from_secs(5 * 24 * 3600),
            retry_schedule: vec![Duration::from_secs(2), Duration::from_secs(4)],
            disable_target_after: Some(3),
        }
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn upserting_creates_the_row_on_first_write() {
        let pool = make_pool().await;
        let org_id = seed_organization(&pool).await;

        let written = with_tx(&pool, async |tx| {
            let mut repo = PgAutomationSettingsRepository::new(&tx);
            repo.upsert(org_id, &custom_settings()).await
        })
        .await
        .unwrap();

        assert_eq!(written, custom_settings());

        let reread = with_tx(&pool, async |tx| {
            let mut repo = PgAutomationSettingsRepository::new(&tx);
            repo.settings_for(org_id).await
        })
        .await
        .unwrap();
        assert_eq!(reread, custom_settings());
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn upserting_again_replaces_the_existing_row_rather_than_conflicting() {
        let pool = make_pool().await;
        let org_id = seed_organization(&pool).await;
        with_tx(&pool, async |tx| {
            let mut repo = PgAutomationSettingsRepository::new(&tx);
            repo.upsert(org_id, &custom_settings()).await
        })
        .await
        .unwrap();

        let second = AutomationSettings {
            event_retention: Duration::from_secs(20 * 24 * 3600),
            ..custom_settings()
        };
        let updated = with_tx(&pool, async |tx| {
            let mut repo = PgAutomationSettingsRepository::new(&tx);
            repo.upsert(org_id, &second).await
        })
        .await
        .unwrap();

        assert_eq!(updated.event_retention, Duration::from_secs(20 * 24 * 3600));

        let reread = with_tx(&pool, async |tx| {
            let mut repo = PgAutomationSettingsRepository::new(&tx);
            repo.settings_for(org_id).await
        })
        .await
        .unwrap();
        assert_eq!(reread, second);
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn upserting_one_organization_leaves_anothers_settings_untouched() {
        let pool = make_pool().await;
        let org_id = seed_organization(&pool).await;
        let other_org = seed_organization(&pool).await;
        with_tx(&pool, async |tx| {
            let mut repo = PgAutomationSettingsRepository::new(&tx);
            repo.upsert(other_org, &custom_settings()).await
        })
        .await
        .unwrap();

        with_tx(&pool, async |tx| {
            let mut repo = PgAutomationSettingsRepository::new(&tx);
            repo.upsert(org_id, &AutomationSettings::default()).await
        })
        .await
        .unwrap();

        let others = with_tx(&pool, async |tx| {
            let mut repo = PgAutomationSettingsRepository::new(&tx);
            repo.settings_for(other_org).await
        })
        .await
        .unwrap();
        assert_eq!(
            others,
            custom_settings(),
            "another organization's row must not move"
        );
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn all_settings_includes_every_configured_organization() {
        let pool = make_pool().await;
        let org_id = seed_organization(&pool).await;
        with_tx(&pool, async |tx| {
            let mut repo = PgAutomationSettingsRepository::new(&tx);
            repo.upsert(org_id, &custom_settings()).await
        })
        .await
        .unwrap();

        let all = with_tx(&pool, async |tx| {
            let mut repo = PgAutomationSettingsRepository::new(&tx);
            repo.all_settings().await
        })
        .await
        .unwrap();

        let mine = all
            .into_iter()
            .find(|(id, _)| *id == org_id)
            .map(|(_, settings)| settings);
        assert_eq!(mine, Some(custom_settings()));
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn all_settings_omits_an_organization_that_never_configured_anything() {
        let pool = make_pool().await;
        let org_id = seed_organization(&pool).await;

        let all = with_tx(&pool, async |tx| {
            let mut repo = PgAutomationSettingsRepository::new(&tx);
            repo.all_settings().await
        })
        .await
        .unwrap();

        assert!(
            all.iter().all(|(id, _)| *id != org_id),
            "an organization with no configured row has nothing to appear in the bulk read; \
             the caller applies the default itself"
        );
    }
}
