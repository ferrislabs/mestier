use common::{CoreError, OrganizationId};
use mestier_macros::repository;
use uuid::Uuid;

use crate::{
    domain::automation::ports::{DeliveryLogRepository, DeliveryRecord},
    infrastructure::postgres::{SharedTx, error::map_sqlx_error},
};

#[repository(domain = DeliveryLog, backend = Postgres)]
pub struct PgDeliveryLogRepository<'tx> {
    tx: SharedTx<'tx>,
}

impl<'tx> PgDeliveryLogRepository<'tx> {
    pub fn new(tx: &SharedTx<'tx>) -> Self {
        Self { tx: tx.clone() }
    }
}

impl<'tx> DeliveryLogRepository for PgDeliveryLogRepository<'tx> {
    async fn list_by_organization(
        &mut self,
        org_id: OrganizationId,
        limit: i64,
        offset: i64,
    ) -> Result<(Vec<DeliveryRecord>, i64), CoreError> {
        let mut tx = self.tx.lock().await;

        let rows = sqlx::query!(
            r#"SELECT d.id, d.event_id, e.name AS event_name, d.status, d.attempts,
                      d.next_attempt_at, d.last_error, d.created_at, d.completed_at
               FROM automation.delivery d
               JOIN automation.event e ON e.id = d.event_id
               WHERE d.org_id = $1
               ORDER BY d.created_at DESC
               LIMIT $2 OFFSET $3"#,
            org_id.0,
            limit,
            offset,
        )
        .fetch_all(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        let total = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM automation.delivery WHERE org_id = $1",
            org_id.0,
        )
        .fetch_one(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?
        .unwrap_or(0);

        let records = rows
            .into_iter()
            .map(|row| DeliveryRecord {
                id: row.id,
                event_id: row.event_id,
                event_name: row.event_name,
                status: row.status,
                attempts: row.attempts,
                next_attempt_at: row.next_attempt_at,
                last_error: row.last_error,
                created_at: row.created_at,
                completed_at: row.completed_at,
            })
            .collect();

        Ok((records, total))
    }

    async fn replay(
        &mut self,
        org_id: OrganizationId,
        delivery_id: Uuid,
    ) -> Result<bool, CoreError> {
        let mut tx = self.tx.lock().await;

        // Scoped by org_id in the WHERE clause rather than checked beforehand:
        // a check-then-act would leave a window, and a mistake here means one
        // organization replaying another's delivery.
        //
        // Attempts reset to zero: a replay is a fresh decision by a human, not
        // the continuation of a schedule that already gave up.
        let replayed = sqlx::query!(
            r#"UPDATE automation.delivery
               SET status = 'pending',
                   attempts = 0,
                   next_attempt_at = now(),
                   last_error = NULL,
                   completed_at = NULL,
                   locked_at = NULL,
                   locked_by = NULL
               WHERE id = $1 AND org_id = $2 AND status IN ('dead', 'failed', 'succeeded')"#,
            delivery_id,
            org_id.0,
        )
        .execute(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?
        .rows_affected();

        Ok(replayed > 0)
    }
}
