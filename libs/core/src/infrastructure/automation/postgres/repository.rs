use common::CoreError;
use events::EventEnvelope;
use mestier_macros::repository;

use crate::{
    domain::automation::ports::EventLogRepository,
    infrastructure::automation::postgres::model::actor_columns,
    infrastructure::postgres::{SharedTx, error::map_sqlx_error},
};

#[repository(domain = EventLog, backend = Postgres)]
pub struct PgEventLogRepository<'tx> {
    tx: SharedTx<'tx>,
}

impl<'tx> PgEventLogRepository<'tx> {
    pub fn new(tx: &SharedTx<'tx>) -> Self {
        Self { tx: tx.clone() }
    }
}

impl<'tx> EventLogRepository for PgEventLogRepository<'tx> {
    async fn append(&mut self, events: &[EventEnvelope]) -> Result<(), CoreError> {
        if events.is_empty() {
            return Ok(());
        }

        // One statement for the whole transaction rather than one per event:
        // a use case that touches several aggregates would otherwise pay a
        // round trip per event, inside the transaction, on the hot path.
        let mut ids = Vec::with_capacity(events.len());
        let mut org_ids = Vec::with_capacity(events.len());
        let mut names = Vec::with_capacity(events.len());
        let mut versions = Vec::with_capacity(events.len());
        let mut subject_kinds = Vec::with_capacity(events.len());
        let mut subject_ids = Vec::with_capacity(events.len());
        let mut payloads = Vec::with_capacity(events.len());
        let mut actor_kinds = Vec::with_capacity(events.len());
        let mut actor_ids = Vec::with_capacity(events.len());
        let mut correlation_ids = Vec::with_capacity(events.len());
        let mut occurred_ats = Vec::with_capacity(events.len());

        for event in events {
            let (actor_kind, actor_id) = actor_columns(event.actor);

            ids.push(event.id);
            org_ids.push(event.org_id.0);
            names.push(event.name.clone());
            // Infallible: the column is INTEGER precisely so this cannot fail.
            versions.push(i32::from(event.version));
            subject_kinds.push(event.subject_kind.clone());
            subject_ids.push(event.subject_id);
            payloads.push(event.payload.clone());
            actor_kinds.push(actor_kind.to_owned());
            actor_ids.push(actor_id);
            correlation_ids.push(event.correlation_id);
            occurred_ats.push(event.occurred_at);
        }

        let mut tx = self.tx.lock().await;
        sqlx::query!(
            r#"
            INSERT INTO automation.event
                (id, org_id, name, version, subject_kind, subject_id, payload,
                 actor_kind, actor_id, correlation_id, occurred_at)
            SELECT * FROM UNNEST(
                $1::uuid[], $2::uuid[], $3::text[], $4::int[], $5::text[], $6::uuid[],
                $7::jsonb[], $8::text[], $9::uuid[], $10::uuid[], $11::timestamptz[]
            )
            "#,
            &ids,
            &org_ids,
            &names,
            &versions,
            &subject_kinds,
            &subject_ids as &[Option<uuid::Uuid>],
            &payloads,
            &actor_kinds,
            &actor_ids as &[Option<uuid::Uuid>],
            &correlation_ids as &[Option<uuid::Uuid>],
            &occurred_ats,
        )
        .execute(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use common::{OrganizationId, generate_uuid_v7};
    use events::{Actor, DomainEvent, EmissionContext, EventSubject};
    use serde_json::{Value, json};
    use sqlx::PgPool;
    use uuid::Uuid;

    use super::*;
    use crate::infrastructure::postgres::with_tx;

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
            .expect("DATABASE_URL must be set to run event log integration tests");
        PgPool::connect(&url).await.unwrap()
    }

    /// The log has a foreign key on `organizations`, so an event needs a real
    /// organization — which needs an owner.
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

    fn envelope(org_id: OrganizationId, quote_id: Uuid, actor: Actor) -> EventEnvelope {
        EventEnvelope::from_event(
            &QuoteAccepted { quote_id },
            &EmissionContext {
                org_id,
                actor,
                correlation_id: None,
            },
        )
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn append_persists_every_field_of_an_envelope() {
        let pool = make_pool().await;
        let org_id = seed_organization(&pool).await;
        let quote_id = generate_uuid_v7();
        let user_id = generate_uuid_v7();
        let written = envelope(org_id, quote_id, Actor::user(user_id));

        with_tx(&pool, async |tx| {
            let mut repo = PgEventLogRepository::new(&tx);
            repo.append(std::slice::from_ref(&written)).await
        })
        .await
        .unwrap();

        let row = sqlx::query!(
            r#"SELECT id, org_id, name, version, subject_kind, subject_id, payload,
                      actor_kind, actor_id, correlation_id, dispatched_at
               FROM automation.event WHERE id = $1"#,
            written.id,
        )
        .fetch_one(&pool)
        .await
        .unwrap();

        assert_eq!(row.org_id, org_id.0);
        assert_eq!(row.name, "quote.accepted");
        assert_eq!(row.version, 1);
        assert_eq!(row.subject_kind, "quote");
        assert_eq!(row.subject_id, Some(quote_id));
        assert_eq!(row.payload, json!({ "quote_id": quote_id }));
        assert_eq!(row.actor_kind, "user");
        assert_eq!(row.actor_id, Some(user_id));
        assert_eq!(row.correlation_id, None);
        assert!(
            row.dispatched_at.is_none(),
            "a freshly appended event is not dispatched"
        );
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn append_writes_a_whole_batch() {
        let pool = make_pool().await;
        let org_id = seed_organization(&pool).await;
        let batch = vec![
            envelope(org_id, generate_uuid_v7(), Actor::system()),
            envelope(org_id, generate_uuid_v7(), Actor::system()),
            envelope(org_id, generate_uuid_v7(), Actor::system()),
        ];

        with_tx(&pool, async |tx| {
            let mut repo = PgEventLogRepository::new(&tx);
            repo.append(&batch).await
        })
        .await
        .unwrap();

        let count = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM automation.event WHERE org_id = $1",
            org_id.0,
        )
        .fetch_one(&pool)
        .await
        .unwrap();

        assert_eq!(count, Some(3));
    }

    /// The invariant the whole chantier rests on, asserted at the lowest level
    /// that can express it: the repository writes inside the caller's
    /// transaction, so a rollback takes the events with it.
    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn a_rolled_back_transaction_leaves_no_event() {
        let pool = make_pool().await;
        let org_id = seed_organization(&pool).await;
        let written = envelope(org_id, generate_uuid_v7(), Actor::system());

        let outcome: Result<(), CoreError> = with_tx(&pool, async |tx| {
            let mut repo = PgEventLogRepository::new(&tx);
            repo.append(std::slice::from_ref(&written)).await?;
            Err(CoreError::Internal("the business rule refused".into()))
        })
        .await;

        assert!(outcome.is_err());

        let count = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM automation.event WHERE id = $1",
            written.id,
        )
        .fetch_one(&pool)
        .await
        .unwrap();

        assert_eq!(
            count,
            Some(0),
            "a rolled back transaction must emit nothing"
        );
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn appending_nothing_is_not_an_error() {
        let pool = make_pool().await;

        let outcome = with_tx(&pool, async |tx| {
            let mut repo = PgEventLogRepository::new(&tx);
            repo.append(&[]).await
        })
        .await;

        assert!(outcome.is_ok());
    }
}
