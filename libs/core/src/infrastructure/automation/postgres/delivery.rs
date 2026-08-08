use std::time::Duration;

use chrono::{DateTime, Utc};
use common::{CoreError, OrganizationId};
use events::EventEnvelope;
use mestier_macros::repository;
use uuid::Uuid;

use crate::{
    domain::automation::{
        ports::{DeliveryRepository, DueDelivery},
        settings::AutomationSettings,
    },
    infrastructure::automation::postgres::model::actor_from_columns,
    infrastructure::postgres::{SharedTx, error::map_sqlx_error},
};

#[repository(domain = Delivery, backend = Postgres)]
pub struct PgDeliveryRepository<'tx> {
    tx: SharedTx<'tx>,
}

impl<'tx> PgDeliveryRepository<'tx> {
    pub fn new(tx: &SharedTx<'tx>) -> Self {
        Self { tx: tx.clone() }
    }
}

impl<'tx> DeliveryRepository for PgDeliveryRepository<'tx> {
    async fn claim_due(
        &mut self,
        worker: &str,
        batch: i64,
        per_org: i64,
    ) -> Result<Vec<DueDelivery>, CoreError> {
        let mut tx = self.tx.lock().await;

        // `SKIP LOCKED` so a second worker takes other rows rather than
        // blocking on the ones already claimed. The whole claim is one
        // statement: selecting then updating separately would leave a window
        // in which two workers both believe they own a delivery.
        let rows = sqlx::query!(
            r#"
            WITH ranked AS (
                SELECT id,
                       row_number() OVER (PARTITION BY org_id ORDER BY next_attempt_at) AS rank
                FROM automation.delivery
                WHERE status IN ('pending', 'failed')
                  AND next_attempt_at <= now()
            ),
            due AS (
                -- Locked by id in a second pass: a window function cannot be
                -- combined with FOR UPDATE. A row claimed by another worker in
                -- between is simply skipped, which is what SKIP LOCKED is for.
                SELECT d.id
                FROM automation.delivery d
                JOIN ranked r ON r.id = d.id
                WHERE r.rank <= $3
                  AND d.status IN ('pending', 'failed')
                  AND d.next_attempt_at <= now()
                ORDER BY d.next_attempt_at
                LIMIT $2
                FOR UPDATE OF d SKIP LOCKED
            ),
            claimed AS (
                UPDATE automation.delivery d
                SET status = 'in_flight', locked_at = now(), locked_by = $1
                FROM due
                WHERE d.id = due.id
                RETURNING d.id, d.org_id, d.subscription_id, d.attempts, d.event_id
            )
            SELECT c.id, c.org_id, c.subscription_id, c.attempts,
                   s.kind, s.target_id,
                   e.id AS event_id, e.name, e.version, e.subject_kind, e.subject_id, e.payload,
                   e.actor_kind, e.actor_id, e.correlation_id, e.occurred_at
            FROM claimed c
            JOIN automation.subscription s ON s.id = c.subscription_id
            JOIN automation.event e ON e.id = c.event_id
            "#,
            worker,
            batch,
            per_org,
        )
        .fetch_all(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        rows.into_iter()
            .map(|row| {
                Ok(DueDelivery {
                    id: row.id,
                    org_id: OrganizationId(row.org_id),
                    subscription_id: row.subscription_id,
                    kind: row.kind,
                    target_id: row.target_id,
                    attempts: u32::try_from(row.attempts).map_err(|_| {
                        CoreError::Internal("delivery attempts went negative".to_owned())
                    })?,
                    event: EventEnvelope {
                        id: row.event_id,
                        org_id: OrganizationId(row.org_id),
                        name: row.name,
                        version: u16::try_from(row.version).map_err(|_| {
                            CoreError::Internal("event version is out of range".to_owned())
                        })?,
                        subject_kind: row.subject_kind,
                        subject_id: row.subject_id,
                        payload: row.payload,
                        actor: actor_from_columns(&row.actor_kind, row.actor_id)?,
                        correlation_id: row.correlation_id,
                        occurred_at: row.occurred_at,
                    },
                })
            })
            .collect()
    }

    async fn settle_succeeded(&mut self, delivery_id: Uuid) -> Result<(), CoreError> {
        let mut tx = self.tx.lock().await;

        let subscription_id = sqlx::query_scalar!(
            r#"
            UPDATE automation.delivery
            SET status = 'succeeded', completed_at = now(), locked_at = NULL, locked_by = NULL
            WHERE id = $1
            RETURNING subscription_id
            "#,
            delivery_id,
        )
        .fetch_one(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        // One success clears the streak: the threshold counts *consecutive*
        // failures, so an endpoint that recovers is not disabled by history.
        sqlx::query!(
            "UPDATE automation.subscription SET consecutive_failures = 0 WHERE id = $1",
            subscription_id,
        )
        .execute(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        Ok(())
    }

    async fn settle_failed(
        &mut self,
        delivery_id: Uuid,
        error: &str,
        next_attempt_at: Option<DateTime<Utc>>,
    ) -> Result<(), CoreError> {
        let mut tx = self.tx.lock().await;

        // `next_attempt_at IS NULL` means the schedule ran out — that is what
        // makes a delivery dead, rather than a separate attempt ceiling that
        // could disagree with the schedule.
        let subscription_id = sqlx::query_scalar!(
            r#"
            UPDATE automation.delivery
            SET attempts = attempts + 1,
                last_error = $2,
                status = CASE WHEN $3::timestamptz IS NULL THEN 'dead' ELSE 'failed' END,
                next_attempt_at = COALESCE($3, next_attempt_at),
                locked_at = NULL,
                locked_by = NULL,
                completed_at = CASE WHEN $3::timestamptz IS NULL THEN now() ELSE NULL END
            WHERE id = $1
            RETURNING subscription_id
            "#,
            delivery_id,
            error,
            next_attempt_at,
        )
        .fetch_one(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        sqlx::query!(
            "UPDATE automation.subscription SET consecutive_failures = consecutive_failures + 1 WHERE id = $1",
            subscription_id,
        )
        .execute(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        Ok(())
    }

    async fn disable_target_if_exhausted(
        &mut self,
        subscription_id: Uuid,
        threshold: u32,
    ) -> Result<bool, CoreError> {
        let mut tx = self.tx.lock().await;

        let disabled = sqlx::query!(
            r#"
            UPDATE automation.subscription
            SET enabled = false, disabled_at = now(), updated_at = now()
            WHERE id = $1 AND enabled AND consecutive_failures >= $2
            "#,
            subscription_id,
            i32::try_from(threshold)
                .map_err(|_| CoreError::Internal("threshold is out of range".to_owned()))?,
        )
        .execute(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?
        .rows_affected();

        Ok(disabled > 0)
    }

    async fn release_stale_claims(&mut self, older_than: DateTime<Utc>) -> Result<u64, CoreError> {
        let mut tx = self.tx.lock().await;

        // Counted as a failed attempt rather than returned untouched: a
        // delivery that reliably kills its worker would otherwise be retried
        // forever. At-least-once already allows a redelivery here, and the
        // receiver deduplicates on the event id.
        let released = sqlx::query!(
            r#"
            UPDATE automation.delivery
            SET status = 'failed',
                attempts = attempts + 1,
                last_error = 'the worker holding this delivery was lost',
                next_attempt_at = now(),
                locked_at = NULL,
                locked_by = NULL
            WHERE status = 'in_flight' AND locked_at < $1
            "#,
            older_than,
        )
        .execute(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?
        .rows_affected();

        Ok(released)
    }

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
        // first read would turn every delivery into a write.
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
    use std::collections::HashSet;

    use chrono::Duration as ChronoDuration;
    use common::generate_uuid_v7;
    use events::{Actor, DomainEvent, EmissionContext, EventEnvelope, EventSubject};
    use serde_json::{Value, json};
    use sqlx::PgPool;

    use super::*;
    use crate::{
        domain::automation::ports::{EventDispatchRepository, EventLogRepository},
        infrastructure::automation::postgres::{
            PgEventDispatchRepository, PgEventLogRepository, dispatcher::DISPATCH_LOCK,
        },
        infrastructure::postgres::with_tx,
    };

    struct QuoteAccepted;

    impl DomainEvent for QuoteAccepted {
        fn name(&self) -> &'static str {
            "quote.accepted"
        }
        fn version(&self) -> u16 {
            1
        }
        fn subject(&self) -> EventSubject {
            EventSubject::new("quote", Uuid::from_u128(7))
        }
        fn payload(&self) -> Value {
            json!({ "hello": "world" })
        }
    }

    async fn make_pool() -> PgPool {
        let url = std::env::var("DATABASE_URL")
            .expect("DATABASE_URL must be set to run delivery integration tests");
        PgPool::connect(&url).await.unwrap()
    }

    /// Seeds an organization with one enabled subscription, emits `count`
    /// events and fans them out — leaving `count` pending deliveries.
    async fn seed_pending(pool: &PgPool, count: usize) -> (OrganizationId, Uuid) {
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

        let subscription_id = generate_uuid_v7();
        sqlx::query!(
            r#"INSERT INTO automation.subscription (id, org_id, kind, target_id, event_names)
               VALUES ($1, $2, 'webhook', $3, ARRAY['quote.accepted'])"#,
            subscription_id,
            org_id,
            generate_uuid_v7(),
        )
        .execute(pool)
        .await
        .unwrap();

        let org = OrganizationId(org_id);
        for _ in 0..count {
            let envelope = EventEnvelope::from_event(
                &QuoteAccepted,
                &EmissionContext {
                    org_id: org,
                    actor: Actor::user(owner_id),
                    correlation_id: None,
                },
            );
            with_tx(pool, async |tx| {
                let mut repo = PgEventLogRepository::new(&tx);
                repo.append(std::slice::from_ref(&envelope)).await
            })
            .await
            .unwrap();
        }

        with_tx(pool, async |tx| {
            let mut repo = PgEventDispatchRepository::new(&tx);
            repo.dispatch_pending(1000).await
        })
        .await
        .unwrap();

        (org, subscription_id)
    }

    async fn claim(pool: &PgPool, worker: &str, batch: i64) -> Vec<DueDelivery> {
        with_tx(pool, async |tx| {
            let mut repo = PgDeliveryRepository::new(&tx);
            repo.claim_due(worker, batch, 1000).await
        })
        .await
        .unwrap()
    }

    async fn status_of(pool: &PgPool, delivery_id: Uuid) -> String {
        sqlx::query_scalar!(
            "SELECT status FROM automation.delivery WHERE id = $1",
            delivery_id,
        )
        .fetch_one(pool)
        .await
        .unwrap()
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn a_claimed_delivery_carries_the_event_it_must_deliver() {
        let _guard = DISPATCH_LOCK.lock().await;
        let pool = make_pool().await;
        let (org_id, subscription_id) = seed_pending(&pool, 1).await;

        let claimed = claim(&pool, "worker-1", 100).await;

        let mine = claimed
            .iter()
            .find(|d| d.org_id == org_id)
            .expect("our delivery was claimed");
        assert_eq!(mine.subscription_id, subscription_id);
        assert_eq!(mine.kind, "webhook");
        assert_eq!(mine.attempts, 0);
        assert_eq!(mine.event.name, "quote.accepted");
        assert_eq!(mine.event.payload, json!({ "hello": "world" }));
        assert!(matches!(mine.event.actor, Actor::User { .. }));
        assert_eq!(status_of(&pool, mine.id).await, "in_flight");
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn a_delivery_that_is_not_due_yet_is_left_alone() {
        let _guard = DISPATCH_LOCK.lock().await;
        let pool = make_pool().await;
        let (org_id, _) = seed_pending(&pool, 1).await;
        sqlx::query!(
            "UPDATE automation.delivery SET next_attempt_at = now() + interval '1 hour' WHERE org_id = $1",
            org_id.0,
        )
        .execute(&pool)
        .await
        .unwrap();

        let claimed = claim(&pool, "worker-1", 100).await;

        assert!(claimed.iter().all(|d| d.org_id != org_id));
    }

    /// Claims must never overlap. Concurrency here may or may not actually
    /// interleave, but a duplicate would fail either way — and `SKIP LOCKED`
    /// is what makes the interleaved case safe rather than blocking.
    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn concurrent_workers_never_claim_the_same_delivery() {
        let _guard = DISPATCH_LOCK.lock().await;
        let pool = make_pool().await;
        seed_pending(&pool, 12).await;

        let (a, b, c) = tokio::join!(
            claim(&pool, "worker-a", 5),
            claim(&pool, "worker-b", 5),
            claim(&pool, "worker-c", 5),
        );

        let all: Vec<Uuid> = a
            .iter()
            .chain(b.iter())
            .chain(c.iter())
            .map(|d| d.id)
            .collect();
        let unique: HashSet<Uuid> = all.iter().copied().collect();
        assert_eq!(all.len(), unique.len(), "a delivery was claimed twice");
    }

    /// One tenant flooding the queue must not starve the others: the claim is
    /// ordered by due date and nothing else, so without a per-organization cap
    /// a single noisy organization fills every batch.
    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn one_organization_cannot_take_the_whole_batch() {
        let _guard = DISPATCH_LOCK.lock().await;
        let pool = make_pool().await;
        let (noisy, _) = seed_pending(&pool, 8).await;
        let (quiet, _) = seed_pending(&pool, 2).await;

        let claimed = with_tx(&pool, async |tx| {
            let mut repo = PgDeliveryRepository::new(&tx);
            repo.claim_due("worker-1", 100, 3).await
        })
        .await
        .unwrap();

        assert_eq!(
            claimed.iter().filter(|d| d.org_id == noisy).count(),
            3,
            "the noisy organization is capped"
        );
        assert_eq!(
            claimed.iter().filter(|d| d.org_id == quiet).count(),
            2,
            "the quiet one still gets served"
        );
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn a_success_finishes_the_delivery_and_clears_the_failure_streak() {
        let _guard = DISPATCH_LOCK.lock().await;
        let pool = make_pool().await;
        let (org_id, subscription_id) = seed_pending(&pool, 1).await;
        sqlx::query!(
            "UPDATE automation.subscription SET consecutive_failures = 3 WHERE id = $1",
            subscription_id,
        )
        .execute(&pool)
        .await
        .unwrap();
        let claimed = claim(&pool, "worker-1", 100).await;
        let mine = claimed.iter().find(|d| d.org_id == org_id).unwrap();

        with_tx(&pool, async |tx| {
            let mut repo = PgDeliveryRepository::new(&tx);
            repo.settle_succeeded(mine.id).await
        })
        .await
        .unwrap();

        assert_eq!(status_of(&pool, mine.id).await, "succeeded");
        let streak = sqlx::query_scalar!(
            "SELECT consecutive_failures FROM automation.subscription WHERE id = $1",
            subscription_id,
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(streak, 0, "a success clears the streak");
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn a_failure_with_a_next_attempt_stays_retryable() {
        let _guard = DISPATCH_LOCK.lock().await;
        let pool = make_pool().await;
        let (org_id, subscription_id) = seed_pending(&pool, 1).await;
        let claimed = claim(&pool, "worker-1", 100).await;
        let mine = claimed.iter().find(|d| d.org_id == org_id).unwrap();

        with_tx(&pool, async |tx| {
            let mut repo = PgDeliveryRepository::new(&tx);
            repo.settle_failed(mine.id, "connection refused", Some(Utc::now()))
                .await
        })
        .await
        .unwrap();

        assert_eq!(status_of(&pool, mine.id).await, "failed");
        let row = sqlx::query!(
            "SELECT attempts, last_error, locked_by FROM automation.delivery WHERE id = $1",
            mine.id,
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(row.attempts, 1);
        assert_eq!(row.last_error.as_deref(), Some("connection refused"));
        assert!(row.locked_by.is_none(), "a settled delivery holds no claim");
        let streak = sqlx::query_scalar!(
            "SELECT consecutive_failures FROM automation.subscription WHERE id = $1",
            subscription_id,
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(streak, 1);
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn an_exhausted_schedule_makes_the_delivery_dead_and_it_is_never_claimed_again() {
        let _guard = DISPATCH_LOCK.lock().await;
        let pool = make_pool().await;
        let (org_id, _) = seed_pending(&pool, 1).await;
        let claimed = claim(&pool, "worker-1", 100).await;
        let mine = claimed.iter().find(|d| d.org_id == org_id).unwrap();

        with_tx(&pool, async |tx| {
            let mut repo = PgDeliveryRepository::new(&tx);
            repo.settle_failed(mine.id, "gave up", None).await
        })
        .await
        .unwrap();

        assert_eq!(status_of(&pool, mine.id).await, "dead");
        let again = claim(&pool, "worker-1", 100).await;
        assert!(again.iter().all(|d| d.id != mine.id));
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn a_target_is_disabled_only_once_it_reaches_the_threshold() {
        let _guard = DISPATCH_LOCK.lock().await;
        let pool = make_pool().await;
        let (_, subscription_id) = seed_pending(&pool, 1).await;
        sqlx::query!(
            "UPDATE automation.subscription SET consecutive_failures = 2 WHERE id = $1",
            subscription_id,
        )
        .execute(&pool)
        .await
        .unwrap();

        let below = with_tx(&pool, async |tx| {
            let mut repo = PgDeliveryRepository::new(&tx);
            repo.disable_target_if_exhausted(subscription_id, 3).await
        })
        .await
        .unwrap();
        assert!(!below, "two failures against a threshold of three");

        sqlx::query!(
            "UPDATE automation.subscription SET consecutive_failures = 3 WHERE id = $1",
            subscription_id,
        )
        .execute(&pool)
        .await
        .unwrap();
        let at = with_tx(&pool, async |tx| {
            let mut repo = PgDeliveryRepository::new(&tx);
            repo.disable_target_if_exhausted(subscription_id, 3).await
        })
        .await
        .unwrap();

        assert!(at);
        let enabled = sqlx::query_scalar!(
            "SELECT enabled FROM automation.subscription WHERE id = $1",
            subscription_id,
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        assert!(!enabled);
    }

    /// A worker that dies mid-flight must not strand its claim: the delivery
    /// has to come back rather than sit `in_flight` forever.
    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn a_lost_workers_claim_comes_back() {
        let _guard = DISPATCH_LOCK.lock().await;
        let pool = make_pool().await;
        let (org_id, _) = seed_pending(&pool, 1).await;
        let claimed = claim(&pool, "worker-doomed", 100).await;
        let mine = claimed.iter().find(|d| d.org_id == org_id).unwrap();
        sqlx::query!(
            "UPDATE automation.delivery SET locked_at = now() - interval '1 hour' WHERE id = $1",
            mine.id,
        )
        .execute(&pool)
        .await
        .unwrap();

        let released = with_tx(&pool, async |tx| {
            let mut repo = PgDeliveryRepository::new(&tx);
            repo.release_stale_claims(Utc::now() - ChronoDuration::minutes(5))
                .await
        })
        .await
        .unwrap();

        assert!(released >= 1);
        let again = claim(&pool, "worker-2", 100).await;
        assert!(
            again.iter().any(|d| d.id == mine.id),
            "the released delivery is claimable again"
        );
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn an_organization_without_a_settings_row_gets_the_defaults() {
        let _guard = DISPATCH_LOCK.lock().await;
        let pool = make_pool().await;
        let (org_id, _) = seed_pending(&pool, 0).await;

        let settings = with_tx(&pool, async |tx| {
            let mut repo = PgDeliveryRepository::new(&tx);
            repo.settings_for(org_id).await
        })
        .await
        .unwrap();

        assert_eq!(settings, AutomationSettings::default());
    }
}
