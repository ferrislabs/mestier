#[cfg(test)]
#[allow(clippy::module_inception)]
mod tests {
    use common::{OrganizationId, generate_uuid_v7};
    use events::{Actor, DomainEvent, EmissionContext, EventEnvelope, EventSubject};
    use serde_json::{Value, json};
    use sqlx::PgPool;
    use uuid::Uuid;

    use crate::application::{MestierUseCase, default_authorizer};
    use crate::domain::automation::ports::EventLogRepository;
    use crate::infrastructure::automation::postgres::PgEventLogRepository;
    use crate::infrastructure::postgres::with_tx;
    use crate::infrastructure::realtime::EventHub;

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
            .expect("DATABASE_URL must be set to run dispatch integration tests");
        PgPool::connect(&url).await.unwrap()
    }

    async fn seed_organization_with_subscription(pool: &PgPool) -> OrganizationId {
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

        sqlx::query!(
            r#"INSERT INTO automation.subscription (id, org_id, kind, target_id, event_names)
               VALUES ($1, $2, 'webhook', $3, ARRAY['quote.accepted'])"#,
            generate_uuid_v7(),
            org_id,
            generate_uuid_v7(),
        )
        .execute(pool)
        .await
        .unwrap();

        OrganizationId(org_id)
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn one_pass_fans_a_committed_event_out_to_its_subscriber() {
        let _guard = crate::infrastructure::automation::postgres::dispatcher::DISPATCH_LOCK
            .lock()
            .await;
        let pool = make_pool().await;
        let org_id = seed_organization_with_subscription(&pool).await;
        let envelope = EventEnvelope::from_event(
            &QuoteAccepted,
            &EmissionContext {
                org_id,
                actor: Actor::system(),
                correlation_id: None,
            },
        );
        with_tx(&pool, async |tx| {
            let mut repo = PgEventLogRepository::new(&tx);
            repo.append(std::slice::from_ref(&envelope)).await
        })
        .await
        .unwrap();
        let usecase = MestierUseCase::new(pool.clone(), default_authorizer(), EventHub::new());

        let outcome = usecase.dispatch_pending_events(100).await.unwrap();

        assert!(outcome.events >= 1);
        let deliveries = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM automation.delivery WHERE event_id = $1",
            envelope.id,
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(deliveries, Some(1));
    }
}
