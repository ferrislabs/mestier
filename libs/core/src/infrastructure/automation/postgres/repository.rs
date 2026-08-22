use chrono::{DateTime, Utc};
use common::{CoreError, OrganizationId};
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

    async fn purge_expired(
        &mut self,
        org_id: OrganizationId,
        older_than: DateTime<Utc>,
        batch: i64,
    ) -> Result<u64, CoreError> {
        let mut tx = self.tx.lock().await;

        // `LIMIT` inside the CTE, then a join back on `id`, is the same
        // bounded-batch shape `PgRunRepository::claim_due` already uses.
        let deleted = sqlx::query!(
            r#"
            WITH victims AS (
                SELECT id FROM automation.event
                WHERE org_id = $1 AND occurred_at < $2
                ORDER BY occurred_at
                LIMIT $3
            )
            DELETE FROM automation.event ev
            USING victims v
            WHERE ev.id = v.id
            "#,
            org_id.0,
            older_than,
            batch,
        )
        .execute(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?
        .rows_affected();

        Ok(deleted)
    }

    async fn organizations_with_automation_data(
        &mut self,
    ) -> Result<Vec<OrganizationId>, CoreError> {
        let mut tx = self.tx.lock().await;

        // Unioned with `automation.run`, not only `automation.event`: an
        // organization whose events already purged clean but still has
        // succeeded runs awaiting their own purge must not be skipped.
        let ids = sqlx::query_scalar!(
            r#"
            SELECT org_id FROM automation.event
            UNION
            SELECT org_id FROM automation.run WHERE status = 'succeeded'
            "#
        )
        .fetch_all(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        Ok(ids.into_iter().flatten().map(OrganizationId).collect())
    }
}

#[cfg(test)]
mod tests {
    use chrono::Utc;
    use common::{OrganizationId, generate_uuid_v7};
    use events::{Actor, DomainEvent, EmissionContext, EventSubject};
    use serde_json::{Value, json};
    use sqlx::PgPool;
    use uuid::Uuid;

    use super::*;
    use crate::application::test_support::automation_pool;
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
        automation_pool().await
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

    /// A stand-in for an event that "happened" `occurred_at` — the shape
    /// every purge test needs, since `append` always writes whatever
    /// `occurred_at` the envelope already carries rather than stamping its
    /// own.
    fn envelope_at(
        org_id: OrganizationId,
        occurred_at: chrono::DateTime<chrono::Utc>,
    ) -> EventEnvelope {
        EventEnvelope {
            occurred_at,
            ..envelope(org_id, generate_uuid_v7(), Actor::system())
        }
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

    // --- purge_expired -------------------------------------------------

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn purging_deletes_events_older_than_the_cutoff_and_keeps_the_rest() {
        let pool = make_pool().await;
        let org_id = seed_organization(&pool).await;
        let cutoff = Utc::now() - chrono::Duration::days(90);
        let expired = envelope_at(org_id, cutoff - chrono::Duration::days(1));
        let fresh = envelope_at(org_id, cutoff + chrono::Duration::days(1));
        with_tx(&pool, async |tx| {
            let mut repo = PgEventLogRepository::new(&tx);
            repo.append(&[expired.clone(), fresh.clone()]).await
        })
        .await
        .unwrap();

        let deleted = with_tx(&pool, async |tx| {
            let mut repo = PgEventLogRepository::new(&tx);
            repo.purge_expired(org_id, cutoff, 100).await
        })
        .await
        .unwrap();

        assert_eq!(deleted, 1);
        let remaining: Vec<Uuid> = sqlx::query_scalar!(
            "SELECT id FROM automation.event WHERE org_id = $1",
            org_id.0,
        )
        .fetch_all(&pool)
        .await
        .unwrap();
        assert_eq!(remaining, vec![fresh.id]);
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn purge_expired_only_touches_the_given_organizations_events() {
        let pool = make_pool().await;
        let org_id = seed_organization(&pool).await;
        let other_org = seed_organization(&pool).await;
        let old = Utc::now() - chrono::Duration::days(100);
        let mine = envelope_at(org_id, old);
        let not_mine = envelope_at(other_org, old);
        with_tx(&pool, async |tx| {
            let mut repo = PgEventLogRepository::new(&tx);
            repo.append(&[mine.clone(), not_mine.clone()]).await
        })
        .await
        .unwrap();

        with_tx(&pool, async |tx| {
            let mut repo = PgEventLogRepository::new(&tx);
            repo.purge_expired(org_id, Utc::now(), 100).await
        })
        .await
        .unwrap();

        let mine_count = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM automation.event WHERE id = $1",
            mine.id,
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(mine_count, Some(0));

        let not_mine_count = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM automation.event WHERE id = $1",
            not_mine.id,
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(
            not_mine_count,
            Some(1),
            "another organization's event must survive a purge scoped to this one"
        );
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn purge_expired_never_deletes_more_than_the_batch_in_one_call() {
        let pool = make_pool().await;
        let org_id = seed_organization(&pool).await;
        let old = Utc::now() - chrono::Duration::days(100);
        let batch = vec![
            envelope_at(org_id, old),
            envelope_at(org_id, old),
            envelope_at(org_id, old),
        ];
        with_tx(&pool, async |tx| {
            let mut repo = PgEventLogRepository::new(&tx);
            repo.append(&batch).await
        })
        .await
        .unwrap();

        let deleted_first_pass = with_tx(&pool, async |tx| {
            let mut repo = PgEventLogRepository::new(&tx);
            repo.purge_expired(org_id, Utc::now(), 2).await
        })
        .await
        .unwrap();
        assert_eq!(deleted_first_pass, 2, "bounded to the batch size");

        let remaining_after_first_pass = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM automation.event WHERE org_id = $1",
            org_id.0,
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(remaining_after_first_pass, Some(1));

        let deleted_second_pass = with_tx(&pool, async |tx| {
            let mut repo = PgEventLogRepository::new(&tx);
            repo.purge_expired(org_id, Utc::now(), 2).await
        })
        .await
        .unwrap();
        assert_eq!(
            deleted_second_pass, 1,
            "the next pass finishes what the first left behind"
        );
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn organizations_with_automation_data_includes_an_organization_with_an_event() {
        let pool = make_pool().await;
        let org_id = seed_organization(&pool).await;
        with_tx(&pool, async |tx| {
            let mut repo = PgEventLogRepository::new(&tx);
            repo.append(&[envelope(org_id, generate_uuid_v7(), Actor::system())])
                .await
        })
        .await
        .unwrap();

        let organizations = with_tx(&pool, async |tx| {
            let mut repo = PgEventLogRepository::new(&tx);
            repo.organizations_with_automation_data().await
        })
        .await
        .unwrap();

        assert!(organizations.contains(&org_id));
    }
}
