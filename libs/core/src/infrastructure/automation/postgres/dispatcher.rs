use common::CoreError;
use mestier_macros::repository;

use crate::{
    domain::automation::ports::{DispatchOutcome, EventDispatchRepository},
    infrastructure::postgres::{SharedTx, error::map_sqlx_error},
};

#[repository(domain = EventDispatch, backend = Postgres)]
pub struct PgEventDispatchRepository<'tx> {
    tx: SharedTx<'tx>,
}

impl<'tx> PgEventDispatchRepository<'tx> {
    pub fn new(tx: &SharedTx<'tx>) -> Self {
        Self { tx: tx.clone() }
    }
}

impl<'tx> EventDispatchRepository for PgEventDispatchRepository<'tx> {
    async fn dispatch_pending(&mut self, batch: i64) -> Result<DispatchOutcome, CoreError> {
        let mut tx = self.tx.lock().await;

        // `SKIP LOCKED` so a second dispatcher works on other events instead of
        // waiting; `FOR UPDATE` so it cannot pick the same ones.
        let claimed: Vec<uuid::Uuid> = sqlx::query_scalar!(
            r#"
            SELECT id
            FROM automation.event
            WHERE dispatched_at IS NULL
            ORDER BY occurred_at
            LIMIT $1
            FOR UPDATE SKIP LOCKED
            "#,
            batch,
        )
        .fetch_all(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        if claimed.is_empty() {
            return Ok(DispatchOutcome::default());
        }

        // Set-based: one statement whatever the batch size, rather than a
        // query per event to find its subscribers.
        //
        // `gen_random_uuid` rather than v7 as everywhere else — Postgres 17
        // has no `uuidv7()`, and generating ids in Rust would mean giving up
        // the single statement. Delivery rows are read by their indexes, not
        // by key order, so nothing depends on it.
        let deliveries = sqlx::query!(
            r#"
            INSERT INTO automation.delivery (id, event_id, subscription_id, org_id)
            SELECT gen_random_uuid(), e.id, s.id, e.org_id
            FROM automation.event e
            JOIN automation.subscription s
              ON s.org_id = e.org_id
             AND s.enabled
             AND e.name = ANY(s.event_names)
            WHERE e.id = ANY($1)
            ON CONFLICT ON CONSTRAINT uq_automation_delivery_event_subscription DO NOTHING
            "#,
            &claimed,
        )
        .execute(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?
        .rows_affected();

        // Marked whether or not anyone wanted them: an event with no
        // subscriber is done with, not re-read on every pass.
        let events = sqlx::query!(
            "UPDATE automation.event SET dispatched_at = now() WHERE id = ANY($1)",
            &claimed,
        )
        .execute(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?
        .rows_affected();

        Ok(DispatchOutcome { events, deliveries })
    }
}

/// Fan-out is global by design — it claims every pending event, not one
/// organization's. Tests that assert on counts therefore cannot run
/// concurrently against a shared database, so they take this lock.
#[cfg(test)]
pub(crate) static DISPATCH_LOCK: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

#[cfg(test)]
mod tests {
    use common::{OrganizationId, generate_uuid_v7};
    use events::{Actor, DomainEvent, EmissionContext, EventEnvelope, EventSubject};
    use serde_json::{Value, json};
    use sqlx::PgPool;
    use uuid::Uuid;

    use super::*;
    use crate::{
        domain::automation::ports::EventLogRepository,
        infrastructure::automation::postgres::PgEventLogRepository,
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
            EventSubject::new("quote", Uuid::from_u128(1))
        }
        fn payload(&self) -> Value {
            json!({})
        }
    }

    async fn make_pool() -> PgPool {
        let url = std::env::var("DATABASE_URL")
            .expect("DATABASE_URL must be set to run dispatcher integration tests");
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

    async fn seed_subscription(
        pool: &PgPool,
        org_id: OrganizationId,
        event_names: &[&str],
        enabled: bool,
    ) -> Uuid {
        let id = generate_uuid_v7();
        let names: Vec<String> = event_names.iter().map(|n| (*n).to_owned()).collect();
        sqlx::query!(
            r#"INSERT INTO automation.subscription (id, org_id, kind, target_id, event_names, enabled)
               VALUES ($1, $2, 'webhook', $3, $4, $5)"#,
            id,
            org_id.0,
            generate_uuid_v7(),
            &names,
            enabled,
        )
        .execute(pool)
        .await
        .unwrap();
        id
    }

    async fn seed_event(pool: &PgPool, org_id: OrganizationId) -> Uuid {
        let envelope = EventEnvelope::from_event(
            &QuoteAccepted,
            &EmissionContext {
                org_id,
                actor: Actor::system(),
                correlation_id: None,
            },
        );
        let id = envelope.id;
        with_tx(pool, async |tx| {
            let mut repo = PgEventLogRepository::new(&tx);
            repo.append(std::slice::from_ref(&envelope)).await
        })
        .await
        .unwrap();
        id
    }

    async fn dispatch(pool: &PgPool) -> DispatchOutcome {
        with_tx(pool, async |tx| {
            let mut repo = PgEventDispatchRepository::new(&tx);
            repo.dispatch_pending(100).await
        })
        .await
        .unwrap()
    }

    async fn deliveries_for(pool: &PgPool, event_id: Uuid) -> i64 {
        sqlx::query_scalar!(
            "SELECT COUNT(*) FROM automation.delivery WHERE event_id = $1",
            event_id,
        )
        .fetch_one(pool)
        .await
        .unwrap()
        .unwrap_or(0)
    }

    async fn is_dispatched(pool: &PgPool, event_id: Uuid) -> bool {
        sqlx::query_scalar!(
            "SELECT dispatched_at IS NOT NULL FROM automation.event WHERE id = $1",
            event_id,
        )
        .fetch_one(pool)
        .await
        .unwrap()
        .unwrap_or(false)
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn an_event_reaches_every_interested_subscription() {
        let _guard = DISPATCH_LOCK.lock().await;
        let pool = make_pool().await;
        let org_id = seed_organization(&pool).await;
        for _ in 0..3 {
            seed_subscription(&pool, org_id, &["quote.accepted"], true).await;
        }
        let event_id = seed_event(&pool, org_id).await;

        dispatch(&pool).await;

        assert_eq!(deliveries_for(&pool, event_id).await, 3);
        assert!(is_dispatched(&pool, event_id).await);
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn a_disabled_subscription_receives_nothing() {
        let _guard = DISPATCH_LOCK.lock().await;
        let pool = make_pool().await;
        let org_id = seed_organization(&pool).await;
        seed_subscription(&pool, org_id, &["quote.accepted"], false).await;
        let event_id = seed_event(&pool, org_id).await;

        dispatch(&pool).await;

        assert_eq!(deliveries_for(&pool, event_id).await, 0);
        assert!(
            is_dispatched(&pool, event_id).await,
            "an event nobody wants is still done with, not re-read forever"
        );
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn a_subscription_of_another_organization_receives_nothing() {
        let _guard = DISPATCH_LOCK.lock().await;
        let pool = make_pool().await;
        let mine = seed_organization(&pool).await;
        let theirs = seed_organization(&pool).await;
        seed_subscription(&pool, theirs, &["quote.accepted"], true).await;
        let event_id = seed_event(&pool, mine).await;

        dispatch(&pool).await;

        assert_eq!(deliveries_for(&pool, event_id).await, 0);
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn a_subscription_to_another_event_receives_nothing() {
        let _guard = DISPATCH_LOCK.lock().await;
        let pool = make_pool().await;
        let org_id = seed_organization(&pool).await;
        seed_subscription(&pool, org_id, &["quote.declined"], true).await;
        let event_id = seed_event(&pool, org_id).await;

        dispatch(&pool).await;

        assert_eq!(deliveries_for(&pool, event_id).await, 0);
    }

    /// Fan-out has to survive being replayed — after a crash, or raced by a
    /// second dispatcher. Re-running it over an event that was already fanned
    /// out must converge, not duplicate.
    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn replaying_the_fan_out_creates_no_duplicate() {
        let _guard = DISPATCH_LOCK.lock().await;
        let pool = make_pool().await;
        let org_id = seed_organization(&pool).await;
        seed_subscription(&pool, org_id, &["quote.accepted"], true).await;
        let event_id = seed_event(&pool, org_id).await;
        dispatch(&pool).await;

        // Simulate a crash between the insert and the mark.
        sqlx::query!(
            "UPDATE automation.event SET dispatched_at = NULL WHERE id = $1",
            event_id,
        )
        .execute(&pool)
        .await
        .unwrap();
        dispatch(&pool).await;

        assert_eq!(deliveries_for(&pool, event_id).await, 1);
        assert!(is_dispatched(&pool, event_id).await);
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn a_dispatched_event_is_not_read_again() {
        let _guard = DISPATCH_LOCK.lock().await;
        let pool = make_pool().await;
        let org_id = seed_organization(&pool).await;
        // Drain whatever earlier tests left behind, so the second pass is
        // measuring this test's event and nothing else.
        dispatch(&pool).await;
        seed_subscription(&pool, org_id, &["quote.accepted"], true).await;
        seed_event(&pool, org_id).await;
        dispatch(&pool).await;

        let second = dispatch(&pool).await;

        assert_eq!(second.events, 0);
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn deleting_an_event_takes_its_deliveries_with_it() {
        let _guard = DISPATCH_LOCK.lock().await;
        let pool = make_pool().await;
        let org_id = seed_organization(&pool).await;
        seed_subscription(&pool, org_id, &["quote.accepted"], true).await;
        let event_id = seed_event(&pool, org_id).await;
        dispatch(&pool).await;

        sqlx::query!("DELETE FROM automation.event WHERE id = $1", event_id)
            .execute(&pool)
            .await
            .unwrap();

        assert_eq!(deliveries_for(&pool, event_id).await, 0);
    }
}
