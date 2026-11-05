use serde_json::{Value, json};

use crate::{client, harness};

async fn create(app: &harness::App, body: Value) -> reqwest::Response {
    client()
        .post(app.workflows_url())
        .bearer_auth(&app.token)
        .json(&body)
        .send()
        .await
        .expect("the api answers the workflow create call")
}

async fn created(app: &harness::App, body: Value) -> Value {
    let response = create(app, body).await;
    assert_eq!(response.status(), 201);
    let envelope: Value = response.json().await.expect("the answer is json");
    envelope["data"].clone()
}

async fn patch(app: &harness::App, workflow: &Value, body: Value) -> reqwest::Response {
    client()
        .patch(app.workflow_url(id_of(workflow)))
        .bearer_auth(&app.token)
        .json(&body)
        .send()
        .await
        .expect("the api answers the workflow update call")
}

async fn patched(app: &harness::App, workflow: &Value, body: Value) -> Value {
    let response = patch(app, workflow, body).await;
    assert_eq!(response.status(), 200);
    let envelope: Value = response.json().await.expect("the answer is json");
    envelope["data"].clone()
}

fn id_of(workflow: &Value) -> &str {
    workflow["id"].as_str().expect("a workflow carries an id")
}

async fn listed_ids(app: &harness::App, query: &str) -> Vec<String> {
    let envelope: Value = client()
        .get(format!("{}{query}", app.workflows_url()))
        .bearer_auth(&app.token)
        .send()
        .await
        .expect("the api answers the workflow list call")
        .json()
        .await
        .expect("the answer is json");
    envelope["data"]
        .as_array()
        .expect("a list")
        .iter()
        .map(|workflow| id_of(workflow).to_owned())
        .collect()
}

async fn save_manual_version(app: &harness::App, workflow: &Value) -> reqwest::Response {
    client()
        .put(app.workflow_versions_url(id_of(workflow)))
        .bearer_auth(&app.token)
        .json(&json!({
            "graph": {
                "connectors": [{
                    "id": "c1",
                    "kind": "flow.condition",
                    "version": 1,
                    "credential_id": null,
                    "config": { "predicate": "{{ true }}" }
                }],
                "edges": [{ "from": "t1", "to": "c1", "branch": null }],
                "triggers": [{ "id": "t1", "kind": "Manual" }]
            },
            "layout": null
        }))
        .send()
        .await
        .expect("the api answers the save version call")
}

async fn start_run(app: &harness::App, workflow: &Value) -> reqwest::Response {
    client()
        .post(app.runs_url(id_of(workflow)))
        .bearer_auth(&app.token)
        .json(&json!({}))
        .send()
        .await
        .expect("the api answers the start run call")
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn the_legacy_create_payload_still_yields_an_enabled_unfiled_untagged_workflow() {
    let app = harness::start().await;

    let workflow = created(&app, json!({ "name": "Legacy", "description": null })).await;

    assert_eq!(workflow["enabled"], true);
    assert_eq!(workflow["folder_id"], Value::Null);
    assert_eq!(workflow["tags"], json!([]));
    assert_eq!(workflow["archived_at"], Value::Null);

    app.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn a_workflow_is_created_disabled_in_a_folder_with_deduplicated_tags() {
    let app = harness::start().await;
    let folder: Value = client()
        .post(app.folders_url())
        .bearer_auth(&app.token)
        .json(&json!({ "name": "Facturation" }))
        .send()
        .await
        .expect("the api answers the folder create call")
        .json()
        .await
        .expect("json");

    let workflow = created(
        &app,
        json!({
            "name": "Draft",
            "enabled": false,
            "folder_id": folder["data"]["id"],
            "tags": [" CRM ", "relance", "crm"]
        }),
    )
    .await;

    assert_eq!(workflow["enabled"], false);
    assert_eq!(workflow["folder_id"], folder["data"]["id"]);
    assert_eq!(workflow["tags"], json!(["CRM", "relance"]));

    let read: Value = client()
        .get(app.workflow_url(id_of(&workflow)))
        .bearer_auth(&app.token)
        .send()
        .await
        .expect("the api answers the workflow read call")
        .json()
        .await
        .expect("json");
    assert_eq!(read["data"]["folder_id"], folder["data"]["id"]);
    assert_eq!(read["data"]["tags"], json!(["CRM", "relance"]));
    assert_eq!(read["data"]["archived_at"], Value::Null);

    app.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn tags_outside_the_limits_are_refused_on_create_and_update() {
    let app = harness::start().await;
    let workflow = created(&app, json!({ "name": "Tagged" })).await;
    let eleven: Vec<String> = (0..11).map(|index| format!("tag-{index}")).collect();

    for tags in [
        json!(eleven),
        json!([""]),
        json!(["   "]),
        json!(["x".repeat(31)]),
    ] {
        let response = create(&app, json!({ "name": "Bad tags", "tags": tags })).await;
        assert_eq!(response.status(), 400, "create with {tags}");
        let body: Value = response.json().await.expect("json");
        assert_eq!(body["code"], "E_VALIDATION");

        let response = patch(&app, &workflow, json!({ "tags": tags })).await;
        assert_eq!(response.status(), 400, "update with {tags}");
    }

    let ten: Vec<String> = (0..10).map(|index| format!("tag-{index}")).collect();
    let updated = patched(&app, &workflow, json!({ "tags": ten })).await;
    assert_eq!(updated["tags"].as_array().expect("tags").len(), 10);

    app.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn a_workflow_in_a_folder_that_does_not_exist_is_refused() {
    let app = harness::start().await;

    let response = create(
        &app,
        json!({ "name": "Misfiled", "folder_id": uuid::Uuid::now_v7() }),
    )
    .await;

    assert_eq!(response.status(), 409);

    app.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn updated_at_moves_for_a_rename_only() {
    let app = harness::start().await;
    let workflow = created(&app, json!({ "name": "Name" })).await;

    let toggled = patched(&app, &workflow, json!({ "enabled": false, "tags": ["a"] })).await;
    assert_eq!(toggled["updated_at"], workflow["updated_at"]);

    let archived = patched(&app, &workflow, json!({ "archived": true })).await;
    assert_eq!(archived["updated_at"], workflow["updated_at"]);

    let renamed = patched(&app, &workflow, json!({ "name": "Other" })).await;
    assert_ne!(renamed["updated_at"], workflow["updated_at"]);

    app.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn archiving_hides_a_workflow_from_the_default_list_and_unarchiving_brings_it_back() {
    let app = harness::start().await;
    let active = created(&app, json!({ "name": "Active" })).await;
    let archived = created(&app, json!({ "name": "Archived" })).await;

    let after = patched(&app, &archived, json!({ "archived": true })).await;
    assert!(after["archived_at"].is_string());
    assert_eq!(after["enabled"], false);

    assert_eq!(listed_ids(&app, "").await, [id_of(&active)]);
    assert_eq!(listed_ids(&app, "?archived=false").await, [id_of(&active)]);
    assert_eq!(listed_ids(&app, "?archived=true").await, [id_of(&archived)]);

    let restored = patched(&app, &archived, json!({ "archived": false })).await;
    assert_eq!(restored["archived_at"], Value::Null);
    assert_eq!(restored["enabled"], false, "unarchiving does not re-enable");
    assert_eq!(listed_ids(&app, "").await.len(), 2);
    assert!(listed_ids(&app, "?archived=true").await.is_empty());

    app.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn an_archived_workflow_refuses_to_be_enabled_started_or_saved() {
    let app = harness::start().await;
    let workflow = created(&app, json!({ "name": "Frozen" })).await;
    assert_eq!(save_manual_version(&app, &workflow).await.status(), 201);
    patched(&app, &workflow, json!({ "archived": true })).await;

    for response in [
        patch(&app, &workflow, json!({ "enabled": true })).await,
        start_run(&app, &workflow).await,
        save_manual_version(&app, &workflow).await,
    ] {
        let status = response.status().as_u16();
        let body: Value = response.json().await.expect("the error is json");
        assert_eq!(
            (status, &body["code"]),
            (409, &json!("E_WORKFLOW_ARCHIVED")),
            "{body}"
        );
        assert_eq!(body["status"], 409);
        assert!(body["message"].is_string());
    }

    patched(&app, &workflow, json!({ "archived": false })).await;
    assert_eq!(save_manual_version(&app, &workflow).await.status(), 201);
    assert_eq!(start_run(&app, &workflow).await.status(), 201);

    app.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn a_disabled_workflow_can_still_be_started_by_hand() {
    let app = harness::start().await;
    let workflow = created(&app, json!({ "name": "Off", "enabled": false })).await;
    assert_eq!(save_manual_version(&app, &workflow).await.status(), 201);

    assert_eq!(start_run(&app, &workflow).await.status(), 201);

    app.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn the_view_only_member_lists_archived_workflows_but_cannot_archive() {
    let app = harness::start().await;
    let workflow = created(&app, json!({ "name": "Mine" })).await;

    let list = client()
        .get(format!("{}?archived=true", app.workflows_url()))
        .bearer_auth(&app.view_only_token)
        .send()
        .await
        .expect("the api answers the view-only list");
    assert_eq!(list.status(), 200);

    let archive = client()
        .patch(app.workflow_url(id_of(&workflow)))
        .bearer_auth(&app.view_only_token)
        .json(&json!({ "archived": true }))
        .send()
        .await
        .expect("the api answers the view-only archive");
    assert_eq!(archive.status(), 403);

    app.cleanup().await;
}
