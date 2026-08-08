use common::{CoreError, OrganizationId, generate_uuid_v7};
use mestier_macros::repository;
use uuid::Uuid;

use crate::{
    domain::automation::ports::SubscriptionRepository,
    infrastructure::postgres::{SharedTx, error::map_sqlx_error},
};

#[repository(domain = Subscription, backend = Postgres)]
pub struct PgSubscriptionRepository<'tx> {
    tx: SharedTx<'tx>,
}

impl<'tx> PgSubscriptionRepository<'tx> {
    pub fn new(tx: &SharedTx<'tx>) -> Self {
        Self { tx: tx.clone() }
    }
}

impl<'tx> SubscriptionRepository for PgSubscriptionRepository<'tx> {
    async fn upsert_for_target(
        &mut self,
        org_id: OrganizationId,
        target_id: Uuid,
        event_names: &[String],
        enabled: bool,
    ) -> Result<(), CoreError> {
        let mut tx = self.tx.lock().await;

        // Replace rather than accumulate: an endpoint has exactly one
        // subscription, and a stale second one would deliver events the user
        // thinks they unsubscribed from.
        sqlx::query!(
            "DELETE FROM automation.subscription WHERE target_id = $1",
            target_id
        )
        .execute(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        sqlx::query!(
            r#"INSERT INTO automation.subscription
                   (id, org_id, kind, target_id, event_names, enabled)
               VALUES ($1, $2, 'webhook', $3, $4, $5)"#,
            generate_uuid_v7(),
            org_id.0,
            target_id,
            event_names,
            enabled,
        )
        .execute(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        Ok(())
    }

    async fn event_names_for_target(&mut self, target_id: Uuid) -> Result<Vec<String>, CoreError> {
        let mut tx = self.tx.lock().await;
        let names = sqlx::query_scalar!(
            "SELECT event_names FROM automation.subscription WHERE target_id = $1",
            target_id,
        )
        .fetch_optional(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        Ok(names.unwrap_or_default())
    }

    async fn delete_for_target(&mut self, target_id: Uuid) -> Result<(), CoreError> {
        let mut tx = self.tx.lock().await;
        sqlx::query!(
            "DELETE FROM automation.subscription WHERE target_id = $1",
            target_id
        )
        .execute(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        Ok(())
    }
}
