use std::collections::BTreeMap;

use mestier_core::{
    Branch, Edge, Graph, OrganizationId, PlacedConnector, PlacedTrigger, Run, RunStatus,
    StepStatus, TriggerKind,
};
use reqwest::{Client, StatusCode};
use serde_json::{Map, Value, json};
use uuid::Uuid;

use crate::harness::{self, App};
use crate::intake::{Intake, id_of, parse, read};

const NOTE: &str = "SECRET-NOTE-TEXT";
const DRAFT: &str = "SECRET-DRAFT-TEXT";
const REPLY: &str = "REPLY-TEXT-TWO";
const FIRST_MESSAGE: &str = "CLIENT-MESSAGE-ONE";
const SECOND_MESSAGE: &str = "CLIENT-MESSAGE-TWO";

fn configuration(value: Value) -> Map<String, Value> {
    match value {
        Value::Object(map) => map,
        other => panic!("a connector configuration is an object, got {other}"),
    }
}

fn connector(id: &str, kind: &str, config: Value) -> PlacedConnector {
    PlacedConnector {
        id: id.to_owned(),
        kind: kind.to_owned(),
        version: 1,
        credential_id: None,
        config: configuration(config),
        name: None,
        description: None,
        disabled: false,
    }
}

fn edge(from: &str, to: &str, branch: Option<Branch>) -> Edge {
    Edge {
        from: from.to_owned(),
        to: to.to_owned(),
        branch,
    }
}

fn reference_graph(members: &[Uuid]) -> Graph {
    let members: Vec<String> = members.iter().map(Uuid::to_string).collect();

    Graph {
        connectors: vec![
            connector(
                "c1",
                "flow.condition",
                json!({
                    "predicate": "{{ default(trigger.ticket.custom_fields.team, \"unknown\") == \"tech\" }}"
                }),
            ),
            connector(
                "c2",
                "mestier.support.assign_ticket",
                json!({ "ticket_id": "{{ trigger.ticket.id }}", "members": members }),
            ),
            connector(
                "c3",
                "flow.condition",
                json!({
                    "predicate": "{{ default(trigger.ticket.custom_fields.priority, \"unknown\") == \"high\" }}"
                }),
            ),
            connector(
                "c4",
                "mestier.support.add_badge",
                json!({ "ticket_id": "{{ trigger.ticket.id }}", "label": "High" }),
            ),
        ],
        edges: vec![
            edge("t1", "c1", None),
            edge("t1", "c3", None),
            edge("c1", "c2", Some(Branch::Then)),
            edge("c3", "c4", Some(Branch::Then)),
        ],
        triggers: vec![PlacedTrigger {
            id: "t1".to_owned(),
            kind: TriggerKind::Events(vec!["ticket.created".to_owned()]),
            name: None,
            description: None,
        }],
    }
}

async fn member_view(app: &App, client: &Client, ticket_id: Uuid) -> Value {
    let response = client
        .get(app.ticket_url(ticket_id, ""))
        .bearer_auth(&app.admin.token)
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body: Value = response.json().await.unwrap();

    body["data"].clone()
}

async fn member_writes(
    app: &App,
    client: &Client,
    ticket_id: Uuid,
    kind: &str,
    body: &str,
) -> Value {
    let response = client
        .post(app.entries_url(ticket_id))
        .bearer_auth(&app.admin.token)
        .json(&json!({ "kind": kind, "body": body }))
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::CREATED, "{kind}");
    let body: Value = response.json().await.unwrap();

    body["data"].clone()
}

fn run_of(runs: &[Run], ticket_id: Uuid) -> &Run {
    runs.iter()
        .find(|run| {
            run.trigger_payload
                .as_ref()
                .is_some_and(|payload| payload["ticket"]["id"] == json!(ticket_id))
        })
        .unwrap_or_else(|| panic!("no run was started for ticket {ticket_id}: {runs:#?}"))
}

async fn steps_of(app: &App, run: &Run) -> BTreeMap<String, (StepStatus, Option<Value>)> {
    app.state
        .usecase
        .list_run_steps(OrganizationId(app.organization_id), run.id)
        .await
        .unwrap()
        .into_iter()
        .map(|step| (step.connector_id, (step.status, step.output)))
        .collect()
}

async fn intake_view(intake: &Intake<'_>, transcript: &mut Vec<String>, ticket_id: Uuid) -> Value {
    let (status, text) = read(intake.view(ticket_id).await).await;
    assert_eq!(status, StatusCode::OK);
    transcript.push(text.clone());

    parse(&text)["data"].clone()
}

fn entries_of(ticket: &Value) -> Vec<(&str, &str)> {
    ticket["entries"]
        .as_array()
        .unwrap()
        .iter()
        .map(|entry| {
            (
                entry["kind"].as_str().unwrap(),
                entry["body"].as_str().unwrap(),
            )
        })
        .collect()
}

fn assert_nothing_internal_leaked(transcript: &[String]) {
    for text in transcript {
        for secret in [NOTE, DRAFT] {
            assert!(!text.contains(secret), "`{secret}` found in {text}");
        }
    }
}

#[tokio::test]
#[ignore = "requires live postgres, redis and s3"]
async fn a_ticket_created_through_intake_is_assigned_badged_and_followed_to_resolution() {
    let app = harness::start().await;
    let client = Client::new();
    let intake = Intake::new(&app).await;
    let mut transcript: Vec<String> = Vec::new();
    app.install_workflow(
        "Triage des tickets",
        reference_graph(&[app.admin.member_id, app.viewer.member_id]),
    )
    .await;

    let (status, text) = read(
        intake
            .create(&json!({
                "subject": "Le déploiement échoue",
                "body": "Le déploiement dpl_8f2a91 échoue.",
                "custom_fields": { "team": "tech", "priority": "high" },
            }))
            .await,
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    transcript.push(text.clone());
    let ticket_id = id_of(&parse(&text)["data"]);
    let runs = app.run_automation_until(1).await;

    assert_eq!(runs.len(), 1);
    let run = run_of(&runs, ticket_id);
    assert_eq!(run.status, RunStatus::Succeeded, "{run:#?}");
    assert_eq!(run.error, None);
    assert_eq!(run.trigger_id.as_deref(), Some("t1"));
    let steps = steps_of(&app, run).await;
    assert_eq!(
        steps.get("c1"),
        Some(&(StepStatus::Succeeded, Some(json!({ "matched": true }))))
    );
    assert_eq!(
        steps.get("c2"),
        Some(&(
            StepStatus::Succeeded,
            Some(json!({
                "ticket_id": ticket_id,
                "assignee_member_id": app.admin.member_id,
            }))
        ))
    );
    assert_eq!(
        steps.get("c3"),
        Some(&(StepStatus::Succeeded, Some(json!({ "matched": true }))))
    );
    assert_eq!(
        steps.get("c4"),
        Some(&(
            StepStatus::Succeeded,
            Some(json!({ "ticket_id": ticket_id, "label": "High", "tone": "neutral" }))
        ))
    );

    let assigned = member_view(&app, &client, ticket_id).await;
    assert_eq!(
        assigned["assignee_member_id"],
        json!(app.admin.member_id.to_string())
    );
    assert!(assigned["assigned_at"].is_string());
    assert_eq!(
        assigned["badges"],
        json!([{ "label": "High", "tone": "neutral" }])
    );
    assert_eq!(
        assigned["custom_fields"],
        json!({ "team": "tech", "priority": "high" })
    );

    let (message_status, message_text) = read(
        intake
            .message(ticket_id, &json!({ "body": FIRST_MESSAGE }))
            .await,
    )
    .await;
    assert_eq!(message_status, StatusCode::CREATED);
    transcript.push(message_text);
    member_writes(&app, &client, ticket_id, "note", NOTE).await;
    let draft = member_writes(&app, &client, ticket_id, "draft", DRAFT).await;
    member_writes(&app, &client, ticket_id, "reply", REPLY).await;

    let thread = member_view(&app, &client, ticket_id).await;
    assert_eq!(thread["id"], json!(ticket_id));
    let members_see = client
        .get(app.entries_url(ticket_id))
        .bearer_auth(&app.admin.token)
        .send()
        .await
        .unwrap();
    let members_see: Value = members_see.json().await.unwrap();
    let member_kinds: Vec<&str> = members_see["data"]
        .as_array()
        .unwrap()
        .iter()
        .map(|entry| entry["kind"].as_str().unwrap())
        .collect();
    assert_eq!(
        member_kinds,
        ["client_message", "internal_note", "draft", "reply"]
    );
    assert_eq!(draft["kind"], "draft");

    let visible = intake_view(&intake, &mut transcript, ticket_id).await;
    assert_eq!(
        entries_of(&visible),
        [("client_message", FIRST_MESSAGE), ("reply", REPLY)]
    );
    let (list_status, list_text) = read(intake.list("").await).await;
    assert_eq!(list_status, StatusCode::OK);
    transcript.push(list_text);
    assert_nothing_internal_leaked(&transcript);

    let (second_status, second_text) = read(
        intake
            .message(ticket_id, &json!({ "body": SECOND_MESSAGE }))
            .await,
    )
    .await;
    assert_eq!(second_status, StatusCode::CREATED);
    transcript.push(second_text.clone());
    let second_message_id = id_of(&parse(&second_text)["data"]);
    let (upload_status, upload_text) = read(
        intake
            .attach(ticket_id, second_message_id, "photo.pdf")
            .await,
    )
    .await;
    assert_eq!(upload_status, StatusCode::CREATED);
    transcript.push(upload_text);

    let with_file = intake_view(&intake, &mut transcript, ticket_id).await;
    assert_eq!(
        entries_of(&with_file),
        [
            ("client_message", FIRST_MESSAGE),
            ("reply", REPLY),
            ("client_message", SECOND_MESSAGE)
        ]
    );
    let attachments = with_file["entries"][2]["attachments"].as_array().unwrap();
    assert_eq!(attachments.len(), 1);
    assert_eq!(attachments[0]["file_name"], "photo.pdf");
    assert_eq!(with_file["entries"][0]["attachments"], json!([]));
    assert_eq!(with_file["entries"][1]["attachments"], json!([]));

    let resolved = client
        .patch(app.ticket_url(ticket_id, "/status"))
        .bearer_auth(&app.admin.token)
        .json(&json!({ "status": "resolved" }))
        .send()
        .await
        .unwrap();
    assert_eq!(resolved.status(), StatusCode::OK);

    let closed = intake_view(&intake, &mut transcript, ticket_id).await;
    assert_eq!(closed["status"], "resolved");
    assert_nothing_internal_leaked(&transcript);

    let (plain_status, plain_text) = read(
        intake
            .create(&json!({
                "subject": "Question de facturation",
                "body": "Où trouver ma dernière facture ?",
                "custom_fields": {},
            }))
            .await,
    )
    .await;
    assert_eq!(plain_status, StatusCode::CREATED);
    let plain_id = id_of(&parse(&plain_text)["data"]);
    let runs = app.run_automation_until(2).await;

    assert_eq!(runs.len(), 2);
    let kept = run_of(&runs, ticket_id);
    assert_eq!(kept.status, RunStatus::Succeeded);
    let plain_run = run_of(&runs, plain_id);
    assert_eq!(plain_run.status, RunStatus::Succeeded, "{plain_run:#?}");
    assert_eq!(plain_run.error, None);
    let plain_steps = steps_of(&app, plain_run).await;
    assert_eq!(
        plain_steps.get("c1"),
        Some(&(StepStatus::Succeeded, Some(json!({ "matched": false }))))
    );
    assert_eq!(
        plain_steps.get("c2").map(|step| step.0),
        Some(StepStatus::Skipped)
    );
    assert_eq!(
        plain_steps.get("c3"),
        Some(&(StepStatus::Succeeded, Some(json!({ "matched": false }))))
    );
    assert_eq!(
        plain_steps.get("c4").map(|step| step.0),
        Some(StepStatus::Skipped)
    );

    let untouched = member_view(&app, &client, plain_id).await;
    assert_eq!(untouched["assignee_member_id"], Value::Null);
    assert_eq!(untouched["assigned_at"], Value::Null);
    assert_eq!(untouched["badges"], json!([]));
    assert_eq!(untouched["status"], "new");
    assert_eq!(untouched["custom_fields"], json!({}));

    let still_assigned = member_view(&app, &client, ticket_id).await;
    assert_eq!(
        still_assigned["assignee_member_id"],
        json!(app.admin.member_id.to_string())
    );
    assert_eq!(
        still_assigned["badges"],
        json!([{ "label": "High", "tone": "neutral" }])
    );
    app.cleanup().await;
}
