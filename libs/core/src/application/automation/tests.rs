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
    use crate::domain::automation::workflow::{Graph, PlacedConnector};
    use crate::infrastructure::automation::postgres::PgEventLogRepository;
    use crate::infrastructure::automation::postgres::dispatcher::DISPATCH_LOCK;
    use crate::infrastructure::postgres::with_tx;

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

    /// A workflow with a saved current version, subscribed to
    /// `quote.accepted` — everything `dispatch_pending_events` needs to turn
    /// a matching event into a run.
    async fn seed_organization_with_workflow_subscription(pool: &PgPool) -> OrganizationId {
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

        let org = OrganizationId(org_id);
        let usecase = MestierUseCase::new(
            pool.clone(),
            default_authorizer(),
            crate::infrastructure::realtime::EventHub::new(),
        );
        let workflow = usecase
            .create_workflow(crate::domain::automation::workflow::CreateWorkflowCommand {
                org_id: org,
                name: "Test workflow".to_string(),
                description: None,
            })
            .await
            .unwrap();
        let mut config = serde_json::Map::new();
        config.insert("predicate".to_string(), json!("{{ true }}"));
        let graph = Graph {
            connectors: vec![PlacedConnector {
                id: "c1".to_string(),
                kind: "flow.condition".to_string(),
                version: 1,
                credential_id: None,
                config,
            }],
            edges: Vec::new(),
        };
        usecase
            .save_workflow_version(
                crate::domain::automation::workflow::SaveWorkflowVersionCommand {
                    org_id: org,
                    workflow_id: workflow.id,
                    graph,
                    created_by: None,
                },
            )
            .await
            .unwrap();

        sqlx::query!(
            r#"INSERT INTO automation.subscription (id, org_id, kind, target_id, event_names)
               VALUES ($1, $2, 'workflow', $3, ARRAY['quote.accepted'])"#,
            generate_uuid_v7(),
            org_id,
            workflow.id,
        )
        .execute(pool)
        .await
        .unwrap();

        org
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn one_pass_fans_a_committed_event_out_to_a_workflow_run() {
        let _guard = DISPATCH_LOCK.lock().await;
        let pool = make_pool().await;
        let org_id = seed_organization_with_workflow_subscription(&pool).await;
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
        let usecase = MestierUseCase::new(
            pool.clone(),
            default_authorizer(),
            crate::infrastructure::realtime::EventHub::new(),
        );

        let outcome = usecase.dispatch_pending_events(100).await.unwrap();

        assert!(outcome.runs >= 1);
        let runs = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM automation.run WHERE trigger_event_id = $1",
            envelope.id,
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(runs, Some(1));
        // Sanity: `next_attempt_at` is set so the run engine actually picks
        // this up, rather than a row nobody will ever claim.
        let due = sqlx::query_scalar!(
            "SELECT next_attempt_at <= now() FROM automation.run WHERE trigger_event_id = $1",
            envelope.id,
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(due, Some(true));
    }
}
