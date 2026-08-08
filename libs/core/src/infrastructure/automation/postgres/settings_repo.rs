use std::time::Duration;

use common::{CoreError, OrganizationId};
use mestier_macros::repository;

use crate::{
    domain::automation::{ports::AutomationSettingsRepository, settings::AutomationSettings},
    infrastructure::postgres::{SharedTx, error::map_sqlx_error},
};

#[repository(domain = AutomationSettings, backend = Postgres)]
pub struct PgAutomationSettingsRepository<'tx> {
    tx: SharedTx<'tx>,
}

impl<'tx> PgAutomationSettingsRepository<'tx> {
    pub fn new(tx: &SharedTx<'tx>) -> Self {
        Self { tx: tx.clone() }
    }
}

fn seconds(duration: Duration) -> i64 {
    // Bounds are validated in the domain, so this is always in range; the
    // saturating conversion is there so an absurd value stored by hand cannot
    // panic a delivery pass.
    i64::try_from(duration.as_secs()).unwrap_or(i64::MAX)
}

impl<'tx> AutomationSettingsRepository for PgAutomationSettingsRepository<'tx> {
    async fn get(&mut self, org_id: OrganizationId) -> Result<AutomationSettings, CoreError> {
        let mut tx = self.tx.lock().await;
        let row = sqlx::query!(
            r#"SELECT event_retention_seconds,
                      succeeded_delivery_retention_seconds,
                      retry_schedule_seconds,
                      disable_target_after
               FROM automation.settings WHERE org_id = $1"#,
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
                .map(|s| Duration::from_secs(s.max(0) as u64))
                .collect(),
            disable_target_after: row.disable_target_after.map(|t| t.max(0) as u32),
        })
    }

    async fn upsert(
        &mut self,
        org_id: OrganizationId,
        settings: &AutomationSettings,
    ) -> Result<AutomationSettings, CoreError> {
        let schedule: Vec<i64> = settings
            .retry_schedule
            .iter()
            .copied()
            .map(seconds)
            .collect();
        let disable_after = settings
            .disable_target_after
            .map(|t| i32::try_from(t).unwrap_or(i32::MAX));

        let mut tx = self.tx.lock().await;
        sqlx::query!(
            r#"INSERT INTO automation.settings
                   (org_id, event_retention_seconds, succeeded_delivery_retention_seconds,
                    retry_schedule_seconds, disable_target_after)
               VALUES ($1, $2, $3, $4, $5)
               ON CONFLICT (org_id) DO UPDATE
               SET event_retention_seconds = EXCLUDED.event_retention_seconds,
                   succeeded_delivery_retention_seconds = EXCLUDED.succeeded_delivery_retention_seconds,
                   retry_schedule_seconds = EXCLUDED.retry_schedule_seconds,
                   disable_target_after = EXCLUDED.disable_target_after,
                   updated_at = now()"#,
            org_id.0,
            seconds(settings.event_retention),
            seconds(settings.succeeded_delivery_retention),
            &schedule,
            disable_after,
        )
        .execute(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        Ok(settings.clone())
    }
}
