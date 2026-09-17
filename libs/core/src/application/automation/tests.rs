#[cfg(test)]
#[allow(clippy::module_inception)]
mod tests {
    use common::{OrganizationId, generate_uuid_v7};
    use events::{Actor, DomainEvent, EmissionContext, EventEnvelope, EventSubject};
    use serde_json::{Value, json};
    use sqlx::PgPool;
    use uuid::Uuid;

    use crate::application::test_support::automation_pool;
    use crate::application::{MestierUseCase, default_authorizer};
    use crate::domain::automation::ports::EventLogRepository;
    use crate::domain::automation::workflow::{
        CreateWorkflowCommand, Edge, Graph, NodePosition, PlacedConnector, PlacedTrigger,
        SaveWorkflowVersionCommand, TriggerKind, WorkflowLayout,
    };
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
        automation_pool().await
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
            edges: vec![Edge {
                from: "t1".to_string(),
                to: "c1".to_string(),
                branch: None,
            }],
            triggers: vec![PlacedTrigger {
                id: "t1".to_string(),
                kind: TriggerKind::Events(vec!["quote.accepted".to_string()]),
            }],
        };
        usecase
            .save_workflow_version(
                crate::domain::automation::workflow::SaveWorkflowVersionCommand {
                    org_id: org,
                    workflow_id: workflow.id,
                    graph,
                    layout: None,
                    created_by: None,
                },
            )
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

    async fn seed_organization(pool: &PgPool, label: &str) -> OrganizationId {
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
            format!("{label} Org"),
            format!("{label}-{org_id}"),
            owner_id,
        )
        .execute(pool)
        .await
        .unwrap();

        OrganizationId(org_id)
    }

    fn layout_with(entries: &[(&str, f64, f64)]) -> WorkflowLayout {
        entries
            .iter()
            .map(|(id, x, y)| ((*id).to_string(), NodePosition { x: *x, y: *y }))
            .collect::<std::collections::BTreeMap<_, _>>()
            .into()
    }

    fn condition_graph(connector_id: &str) -> Graph {
        let mut config = serde_json::Map::new();
        config.insert("predicate".to_string(), json!("{{ true }}"));
        Graph {
            connectors: vec![PlacedConnector {
                id: connector_id.to_string(),
                kind: "flow.condition".to_string(),
                version: 1,
                credential_id: None,
                config,
            }],
            edges: vec![Edge {
                from: "t1".to_string(),
                to: connector_id.to_string(),
                branch: None,
            }],
            triggers: vec![PlacedTrigger {
                id: "t1".to_string(),
                kind: TriggerKind::Manual,
            }],
        }
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn a_layout_saved_with_a_graph_comes_back_identical_from_the_workflow_detail_read() {
        let pool = make_pool().await;
        let org_id = seed_organization(&pool, "layout-round-trip").await;
        let usecase = MestierUseCase::new(
            pool.clone(),
            default_authorizer(),
            crate::infrastructure::realtime::EventHub::new(),
        );
        let workflow = usecase
            .create_workflow(CreateWorkflowCommand {
                org_id,
                name: "Layout round trip".to_string(),
                description: None,
            })
            .await
            .unwrap();
        let layout = layout_with(&[("c1", 10.5, -20.0)]);

        usecase
            .save_workflow_version(SaveWorkflowVersionCommand {
                org_id,
                workflow_id: workflow.id,
                graph: condition_graph("c1"),
                layout: Some(layout.clone()),
                created_by: None,
            })
            .await
            .unwrap();

        let found = usecase.find_workflow(org_id, workflow.id).await.unwrap();

        assert_eq!(found.unwrap().layout, Some(layout));
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn saving_a_graph_with_no_layout_leaves_a_previously_stored_layout_untouched() {
        let pool = make_pool().await;
        let org_id = seed_organization(&pool, "layout-untouched").await;
        let usecase = MestierUseCase::new(
            pool.clone(),
            default_authorizer(),
            crate::infrastructure::realtime::EventHub::new(),
        );
        let workflow = usecase
            .create_workflow(CreateWorkflowCommand {
                org_id,
                name: "Layout untouched".to_string(),
                description: None,
            })
            .await
            .unwrap();
        let layout = layout_with(&[("c1", 1.0, 2.0)]);
        usecase
            .save_workflow_version(SaveWorkflowVersionCommand {
                org_id,
                workflow_id: workflow.id,
                graph: condition_graph("c1"),
                layout: Some(layout.clone()),
                created_by: None,
            })
            .await
            .unwrap();

        usecase
            .save_workflow_version(SaveWorkflowVersionCommand {
                org_id,
                workflow_id: workflow.id,
                graph: condition_graph("c1"),
                layout: None,
                created_by: None,
            })
            .await
            .unwrap();

        let found = usecase.find_workflow(org_id, workflow.id).await.unwrap();

        assert_eq!(found.unwrap().layout, Some(layout));
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn a_layout_naming_a_connector_absent_from_the_graph_is_accepted_and_stored() {
        let pool = make_pool().await;
        let org_id = seed_organization(&pool, "layout-stale-entry").await;
        let usecase = MestierUseCase::new(
            pool.clone(),
            default_authorizer(),
            crate::infrastructure::realtime::EventHub::new(),
        );
        let workflow = usecase
            .create_workflow(CreateWorkflowCommand {
                org_id,
                name: "Layout with a deleted connector".to_string(),
                description: None,
            })
            .await
            .unwrap();
        let layout = layout_with(&[("c1", 1.0, 2.0), ("deleted-in-editor", 3.0, 4.0)]);

        usecase
            .save_workflow_version(SaveWorkflowVersionCommand {
                org_id,
                workflow_id: workflow.id,
                graph: condition_graph("c1"),
                layout: Some(layout.clone()),
                created_by: None,
            })
            .await
            .unwrap();

        let found = usecase.find_workflow(org_id, workflow.id).await.unwrap();

        assert_eq!(found.unwrap().layout, Some(layout));
    }
}
