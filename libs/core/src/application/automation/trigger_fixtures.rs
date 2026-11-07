use common::{CoreError, OrganizationId, generate_uuid_v7};
use serde_json::json;
use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::automation::ports::WorkflowRepository;
use crate::domain::automation::workflow::{
    ContractKey, ContractKind, Edge, Frequency, Graph, OnInvalid, PlacedConnector, PlacedTrigger,
    TriggerKind, WebhookMethod, Weekday, Workflow,
};
use crate::infrastructure::automation::postgres::PgWorkflowRepository;
use crate::infrastructure::postgres::with_tx;

pub(crate) async fn seed_organization(pool: &PgPool, label: &str) -> OrganizationId {
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

pub(crate) async fn set_organization_timezone(
    pool: &PgPool,
    org_id: OrganizationId,
    timezone: &str,
) {
    sqlx::query!(
        "UPDATE organizations SET timezone = $2 WHERE id = $1",
        org_id.0,
        timezone,
    )
    .execute(pool)
    .await
    .unwrap();
}

pub(crate) fn condition(id: &str) -> PlacedConnector {
    let mut config = serde_json::Map::new();
    config.insert("predicate".to_string(), json!("{{ true }}"));
    PlacedConnector {
        id: id.to_string(),
        kind: "flow.condition".to_string(),
        version: 1,
        credential_id: None,
        config,
        name: None,
        description: None,
        disabled: false,
        operation: None,
    }
}

pub(crate) fn graph_of(triggers: Vec<(&str, TriggerKind)>) -> Graph {
    Graph {
        connectors: vec![condition("c1")],
        edges: triggers
            .iter()
            .map(|(id, _)| Edge {
                from: (*id).to_string(),
                to: "c1".to_string(),
                branch: None,
            })
            .collect(),
        triggers: triggers
            .into_iter()
            .map(|(id, kind)| PlacedTrigger {
                id: id.to_string(),
                kind,
                name: None,
                description: None,
            })
            .collect(),
    }
}

pub(crate) fn webhook(method: WebhookMethod, path: &str) -> TriggerKind {
    TriggerKind::Webhook {
        method,
        path: path.to_string(),
        credential_id: None,
        contract: Vec::new(),
        on_invalid: OnInvalid::Reject,
    }
}

pub(crate) fn webhook_with(
    path: &str,
    credential_id: Option<Uuid>,
    contract: Vec<ContractKey>,
    on_invalid: OnInvalid,
) -> TriggerKind {
    TriggerKind::Webhook {
        method: WebhookMethod::Post,
        path: path.to_string(),
        credential_id,
        contract,
        on_invalid,
    }
}

pub(crate) fn key(name: &str, kind: ContractKind, required: bool) -> ContractKey {
    ContractKey {
        name: name.to_string(),
        kind,
        required,
        example: String::new(),
    }
}

pub(crate) fn daily(time: &str, timezone: Option<&str>) -> TriggerKind {
    TriggerKind::Schedule {
        frequency: Frequency::Daily,
        time: time.to_string(),
        weekday: None,
        timezone: timezone.map(str::to_string),
    }
}

pub(crate) fn weekly(weekday: Weekday, time: &str, timezone: Option<&str>) -> TriggerKind {
    TriggerKind::Schedule {
        frequency: Frequency::Weekly,
        time: time.to_string(),
        weekday: Some(weekday),
        timezone: timezone.map(str::to_string),
    }
}

pub(crate) async fn seed_workflow(pool: &PgPool, org_id: OrganizationId, graph: &Graph) -> Uuid {
    with_tx(pool, async |tx| {
        let mut repo = PgWorkflowRepository::new(&tx);
        let now = chrono::Utc::now();
        let workflow = repo
            .insert(&Workflow {
                id: generate_uuid_v7(),
                org_id,
                name: "Trigger test workflow".to_string(),
                description: None,
                enabled: true,
                current_version_id: None,
                layout: None,
                folder_id: None,
                tags: Default::default(),
                archived_at: None,
                created_at: now,
                updated_at: now,
            })
            .await?;
        repo.insert_version(org_id, workflow.id, graph, None)
            .await?;
        Ok::<_, CoreError>(workflow.id)
    })
    .await
    .unwrap()
}

pub(crate) async fn set_enabled(pool: &PgPool, workflow_id: Uuid, enabled: bool) {
    sqlx::query!(
        "UPDATE automation.workflow SET enabled = $2 WHERE id = $1",
        workflow_id,
        enabled,
    )
    .execute(pool)
    .await
    .unwrap();
}

pub(crate) async fn archive(pool: &PgPool, workflow_id: Uuid) {
    sqlx::query!(
        "UPDATE automation.workflow SET enabled = false, archived_at = now() WHERE id = $1",
        workflow_id,
    )
    .execute(pool)
    .await
    .unwrap();
}
