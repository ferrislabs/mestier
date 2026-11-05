use serde_json::{Value, json};
use uuid::Uuid;

use crate::{client, harness};

const NEW_RUN_FIELDS: [&str; 7] = [
    "number",
    "steps_done",
    "steps_total",
    "trigger_id",
    "trigger_event_name",
    "failed_connector_id",
    "workflow_version_created_at",
];

fn graph_of_two_connectors() -> Value {
    json!({
        "connectors": [
            {
                "id": "c1", "kind": "flow.condition", "version": 1,
                "credential_id": null, "config": { "predicate": "{{ true }}" }
            },
            {
                "id": "c2", "kind": "flow.condition", "version": 1,
                "credential_id": null, "config": { "predicate": "{{ true }}" }
            }
        ],
        "edges": [
            { "from": "t1", "to": "c1", "branch": null },
            { "from": "c1", "to": "c2", "branch": "Then" }
        ],
        "triggers": [{ "id": "t1", "kind": "Manual" }]
    })
}

async fn workflow_with_a_version(
    workflows_url: &str,
    token: &str,
    version_url: impl Fn(&str) -> String,
) -> String {
    let created: Value = client()
        .post(workflows_url)
        .bearer_auth(token)
        .json(&json!({ "name": "Numbered" }))
        .send()
        .await
        .expect("the api answers the create call")
        .json()
        .await
        .expect("json");
    let workflow_id = created["data"]["id"].as_str().expect("an id").to_owned();
    let saved = client()
        .put(version_url(&workflow_id))
        .bearer_auth(token)
        .json(&json!({ "graph": graph_of_two_connectors(), "layout": null }))
        .send()
        .await
        .expect("the api answers the save call");
    assert_eq!(saved.status(), 201);
    workflow_id
}

async fn main_workflow(app: &harness::App) -> String {
    workflow_with_a_version(&app.workflows_url(), &app.token, |id| {
        app.workflow_versions_url(id)
    })
    .await
}

async fn start(app: &harness::App, workflow_id: &str) -> String {
    let started: Value = client()
        .post(app.runs_url(workflow_id))
        .bearer_auth(&app.token)
        .json(&json!({}))
        .send()
        .await
        .expect("the api answers the start call")
        .json()
        .await
        .expect("json");
    started["data"]["run_id"]
        .as_str()
        .expect("a run id")
        .to_owned()
}

async fn get_run(app: &harness::App, run_id: &str) -> Value {
    let read: Value = client()
        .get(app.run_url(run_id))
        .bearer_auth(&app.token)
        .send()
        .await
        .expect("the api answers the run read")
        .json()
        .await
        .expect("json");
    read["data"].clone()
}

async fn insert_step(app: &harness::App, run_id: &str, connector_id: &str, status: &str) {
    sqlx::query(
        "INSERT INTO automation.run_step (id, run_id, connector_id, iteration_path, attempts, status)
         VALUES ($1, $2, $3, '', 1, $4)",
    )
    .bind(Uuid::now_v7())
    .bind(run_id.parse::<Uuid>().expect("a uuid"))
    .bind(connector_id)
    .bind(status)
    .execute(&app.pool)
    .await
    .expect("insert the step");
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn runs_are_numbered_per_organization_and_listed_most_recent_first_with_their_new_fields() {
    let app = harness::start().await;
    let workflow_id = main_workflow(&app).await;
    let first = start(&app, &workflow_id).await;
    let second = start(&app, &workflow_id).await;
    let other_workflow = workflow_with_a_version(
        &app.other_organization_workflows_url(),
        &app.outsider_token,
        |id| format!("{}/{id}/versions", app.other_organization_workflows_url()),
    )
    .await;
    let other_run: Value = client()
        .post(format!(
            "{}/{other_workflow}/runs",
            app.other_organization_workflows_url()
        ))
        .bearer_auth(&app.outsider_token)
        .json(&json!({}))
        .send()
        .await
        .expect("the api answers the other organization's start")
        .json()
        .await
        .expect("json");
    assert!(other_run["data"]["run_id"].is_string());

    let listed: Value = client()
        .get(app.run_log_url())
        .bearer_auth(&app.token)
        .send()
        .await
        .expect("the api answers the list call")
        .json()
        .await
        .expect("json");

    let runs = listed["data"].as_array().expect("a list");
    assert_eq!(runs.len(), 2, "the other organization's run is not listed");
    assert_eq!(runs[0]["id"], second.as_str());
    assert_eq!(runs[0]["number"], 2);
    assert_eq!(runs[1]["id"], first.as_str());
    assert_eq!(runs[1]["number"], 1);
    for run in runs {
        for field in NEW_RUN_FIELDS {
            assert!(run.get(field).is_some(), "{field} missing: {run}");
        }
        assert_eq!(run["steps_done"], 0);
        assert_eq!(run["steps_total"], 2);
        assert_eq!(run["trigger_id"], Value::Null);
        assert_eq!(run["trigger_event_name"], Value::Null);
        assert_eq!(run["failed_connector_id"], Value::Null);
        assert!(run["workflow_version_created_at"].is_string());
    }

    app.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn the_run_detail_repeats_the_new_fields_and_reflects_its_steps() {
    let app = harness::start().await;
    let workflow_id = main_workflow(&app).await;
    let run_id = start(&app, &workflow_id).await;
    insert_step(&app, &run_id, "c1", "succeeded").await;
    insert_step(&app, &run_id, "c2", "dead").await;

    let detail = get_run(&app, &run_id).await;

    for field in NEW_RUN_FIELDS {
        assert!(detail.get(field).is_some(), "{field} missing: {detail}");
    }
    assert_eq!(detail["number"], 1);
    assert_eq!(detail["steps_done"], 1);
    assert_eq!(detail["steps_total"], 2);
    assert_eq!(detail["failed_connector_id"], "c2");
    assert_eq!(detail["steps"].as_array().expect("steps").len(), 2);

    app.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn replaying_a_run_keeps_its_number_and_later_runs_carry_on_counting() {
    let app = harness::start().await;
    let workflow_id = main_workflow(&app).await;
    let first = start(&app, &workflow_id).await;
    let second = start(&app, &workflow_id).await;
    insert_step(&app, &first, "c1", "succeeded").await;
    sqlx::query("UPDATE automation.run SET status = 'failed' WHERE id = $1")
        .bind(first.parse::<Uuid>().expect("a uuid"))
        .execute(&app.pool)
        .await
        .expect("finish the first run");

    let replayed = client()
        .post(app.run_replay_url(&first))
        .bearer_auth(&app.token)
        .json(&json!({ "connector_id": "c1" }))
        .send()
        .await
        .expect("the api answers the replay");
    assert_eq!(replayed.status(), 200);
    let replayed: Value = replayed.json().await.expect("json");

    assert_eq!(replayed["data"]["id"], first.as_str());
    assert_eq!(replayed["data"]["number"], 1);
    assert_eq!(replayed["data"]["status"], "pending");
    assert_eq!(get_run(&app, &second).await["number"], 2);
    let third = start(&app, &workflow_id).await;
    assert_eq!(get_run(&app, &third).await["number"], 3);

    app.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn a_run_of_another_organization_reads_back_not_found() {
    let app = harness::start().await;
    let workflow_id = main_workflow(&app).await;
    let run_id = start(&app, &workflow_id).await;

    let stranger_view = client()
        .get(format!(
            "{}/api/v1/organizations/{}/automation/runs/{run_id}",
            app.base_url,
            app.other_organization_id()
        ))
        .bearer_auth(&app.outsider_token)
        .send()
        .await
        .expect("the api answers");

    assert_eq!(stranger_view.status(), 404);

    app.cleanup().await;
}
