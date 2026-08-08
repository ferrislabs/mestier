use std::sync::Arc;

use common::CoreError;
use sqlx::{PgPool, Postgres, Transaction};
use tokio::sync::{Mutex, MutexGuard};

use crate::domain::automation::ports::EventLogRepository;
use crate::infrastructure::automation::TransactionalEventEmitter;
use crate::infrastructure::postgres::error::map_sqlx_error;
use crate::infrastructure::registry::{Backend, RepoFor, domain};

/// Cloneable handle over a live transaction so several repositories can share
/// it without juggling exclusive `&mut` borrows. Each repository takes its own
/// clone and locks the inner transaction only for the duration of one query.
/// Contention is effectively zero (a use-case runs sequentially on one task)
/// — the mutex exists to satisfy the borrow checker while keeping the
/// hexagonal split: the service stays unaware of `tx`.
pub struct SharedTx<'tx> {
    inner: Arc<Mutex<&'tx mut Transaction<'static, Postgres>>>,
}

impl<'tx> SharedTx<'tx> {
    fn new(tx: &'tx mut Transaction<'static, Postgres>) -> Self {
        Self {
            inner: Arc::new(Mutex::new(tx)),
        }
    }

    pub async fn lock(&self) -> MutexGuard<'_, &'tx mut Transaction<'static, Postgres>> {
        self.inner.lock().await
    }
}

impl<'tx> Clone for SharedTx<'tx> {
    fn clone(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
        }
    }
}

/// Runs `work` in a transaction, then writes whatever it emitted to the event
/// log **inside that same transaction**, and only then commits.
///
/// The append happens here rather than at the end of the caller's body on
/// purpose: a body that returns early — `?` on an error, or an explicit
/// `return Ok(..)` — would otherwise skip it. On the error path the events go
/// down with the rollback, which is the invariant the automation backbone
/// rests on: an event exists if and only if the transaction that produced it
/// committed.
///
/// `NOTIFY` is fired after the commit, and only when something was written.
/// It carries no payload: it is a wake-up hint for the dispatcher, which finds
/// its work by querying `dispatched_at IS NULL`. A missed notification costs a
/// poll interval, never an event.
pub async fn with_tx_emitting<F, T>(
    pool: &PgPool,
    emitter: &TransactionalEventEmitter,
    work: F,
) -> Result<T, CoreError>
where
    F: AsyncFnOnce(SharedTx<'_>) -> Result<T, CoreError>,
{
    let mut tx = pool.begin().await.map_err(map_sqlx_error)?;

    let outcome = {
        let shared = SharedTx::new(&mut tx);
        match work(shared.clone()).await {
            Ok(value) => {
                let events = emitter.drain()?;
                let appended = events.len();
                let mut repository = <Backend as RepoFor<domain::EventLog>>::build(&shared);
                repository.append(&events).await.map(|()| (value, appended))
            }
            Err(err) => Err(err),
        }
    };

    match outcome {
        Ok((value, appended)) => {
            tx.commit().await.map_err(map_sqlx_error)?;
            if appended > 0 {
                notify_dispatcher(pool).await;
            }
            Ok(value)
        }
        Err(err) => {
            let _ = tx.rollback().await;
            Err(err)
        }
    }
}

/// Best-effort: a failed notification only delays the dispatcher until its
/// next poll, so it must never turn a committed transaction into an error the
/// caller sees.
async fn notify_dispatcher(pool: &PgPool) {
    if let Err(error) = sqlx::query("NOTIFY automation_event").execute(pool).await {
        tracing::warn!(%error, "failed to notify the automation dispatcher");
    }
}

pub async fn with_tx<F, T>(pool: &PgPool, work: F) -> Result<T, CoreError>
where
    F: AsyncFnOnce(SharedTx<'_>) -> Result<T, CoreError>,
{
    let mut tx = pool.begin().await.map_err(map_sqlx_error)?;

    let result = {
        let shared = SharedTx::new(&mut tx);
        work(shared).await
        // all `SharedTx` clones drop here, releasing the &mut borrow on `tx`
    };

    match result {
        Ok(value) => {
            tx.commit().await.map_err(map_sqlx_error)?;
            Ok(value)
        }
        Err(err) => {
            let _ = tx.rollback().await;
            Err(err)
        }
    }
}

#[cfg(test)]
mod tests {
    use common::{OrganizationId, generate_uuid_v7};
    use events::{Actor, DomainEvent, EventSubject};
    use serde_json::{Value, json};
    use uuid::Uuid;

    use super::*;

    struct QuoteAccepted {
        quote_id: Uuid,
    }

    impl DomainEvent for QuoteAccepted {
        fn name(&self) -> &'static str {
            "quote.accepted"
        }

        fn version(&self) -> u16 {
            1
        }

        fn subject(&self) -> EventSubject {
            EventSubject::new("quote", self.quote_id)
        }

        fn payload(&self) -> Value {
            json!({ "quote_id": self.quote_id })
        }
    }

    async fn make_pool() -> PgPool {
        let url = std::env::var("DATABASE_URL")
            .expect("DATABASE_URL must be set to run transaction integration tests");
        PgPool::connect(&url).await.unwrap()
    }

    async fn seed_owner(pool: &PgPool) -> Uuid {
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
        owner_id
    }

    async fn seed_organization(pool: &PgPool) -> OrganizationId {
        let owner_id = seed_owner(pool).await;
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

    async fn event_count(pool: &PgPool, org_id: OrganizationId) -> i64 {
        sqlx::query_scalar!(
            "SELECT COUNT(*) FROM automation.event WHERE org_id = $1",
            org_id.0,
        )
        .fetch_one(pool)
        .await
        .unwrap()
        .unwrap_or(0)
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn a_committed_transaction_writes_what_it_emitted() {
        let pool = make_pool().await;
        let org_id = seed_organization(&pool).await;
        let emitter = TransactionalEventEmitter::new(Actor::system(), None);

        with_tx_emitting(&pool, &emitter, async |_tx| {
            emitter.emit(
                org_id,
                &QuoteAccepted {
                    quote_id: generate_uuid_v7(),
                },
            )
        })
        .await
        .unwrap();

        assert_eq!(event_count(&pool, org_id).await, 1);
    }

    /// The invariant of the whole chantier, through the real seam: the business
    /// write and the event it produced disappear together.
    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn a_rolled_back_transaction_writes_neither_the_event_nor_the_business_row() {
        let pool = make_pool().await;
        let org_id = seed_organization(&pool).await;
        let owner_id = seed_owner(&pool).await;
        let doomed_org = generate_uuid_v7();
        let emitter = TransactionalEventEmitter::new(Actor::system(), None);

        let outcome: Result<(), CoreError> = with_tx_emitting(&pool, &emitter, async |tx| {
            let mut guard = tx.lock().await;
            sqlx::query!(
                r#"INSERT INTO organizations (id, name, slug, owner_id)
                   VALUES ($1, $2, $3, $4)"#,
                doomed_org,
                "Doomed Org",
                format!("doomed-{doomed_org}"),
                owner_id,
            )
            .execute(&mut ***guard)
            .await
            .unwrap();
            drop(guard);

            emitter.emit(
                org_id,
                &QuoteAccepted {
                    quote_id: generate_uuid_v7(),
                },
            )?;

            Err(CoreError::Internal("the business rule refused".into()))
        })
        .await;

        assert!(outcome.is_err());
        assert_eq!(
            event_count(&pool, org_id).await,
            0,
            "a rolled back transaction must leave no event"
        );

        let survived = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM organizations WHERE id = $1",
            doomed_org,
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(survived, Some(0), "the business write rolled back too");
    }

    /// A body that returns early must not be able to commit while dropping its
    /// events — which is exactly what appending at the end of the macro's body
    /// would have allowed.
    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn an_early_return_still_persists_what_was_emitted() {
        let pool = make_pool().await;
        let org_id = seed_organization(&pool).await;
        let emitter = TransactionalEventEmitter::new(Actor::system(), None);

        let stop_early = true;

        with_tx_emitting(&pool, &emitter, async |_tx| {
            emitter.emit(
                org_id,
                &QuoteAccepted {
                    quote_id: generate_uuid_v7(),
                },
            )?;

            if stop_early {
                return Ok(());
            }

            emitter.emit(
                org_id,
                &QuoteAccepted {
                    quote_id: generate_uuid_v7(),
                },
            )?;
            Ok(())
        })
        .await
        .unwrap();

        assert_eq!(
            event_count(&pool, org_id).await,
            1,
            "the event emitted before the early return is persisted, and only it"
        );
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn a_transaction_that_emits_nothing_still_commits() {
        let pool = make_pool().await;
        let org_id = seed_organization(&pool).await;
        let emitter = TransactionalEventEmitter::new(Actor::system(), None);

        let outcome = with_tx_emitting(&pool, &emitter, async |_tx| Ok(())).await;

        assert!(outcome.is_ok());
        assert_eq!(event_count(&pool, org_id).await, 0);
    }
}
