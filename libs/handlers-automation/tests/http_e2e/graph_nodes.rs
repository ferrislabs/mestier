use serde_json::{Value, json};

use crate::{client, harness};

async fn new_workflow(app: &harness::App) -> String {
    let created: Value = client()
        .post(app.workflows_url())
        .bearer_auth(&app.token)
        .json(&json!({ "name": "Graph nodes" }))
        .send()
        .await
        .expect("the api answers the create call")
        .json()
        .await
        .expect("json");
    created["data"]["id"].as_str().expect("an id").to_owned()
}

fn graph(connector_extra: Value, second_name: Option<&str>) -> Value {
    let mut first = json!({
        "id": "c1",
        "kind": "flow.condition",
        "version": 1,
        "credential_id": null,
        "config": { "predicate": "{{ true }}" }
    });
    first
        .as_object_mut()
        .expect("an object")
        .extend(connector_extra.as_object().expect("an object").clone());
    let mut second = json!({
        "id": "c2",
        "kind": "flow.condition",
        "version": 1,
        "credential_id": null,
        "config": { "predicate": "{{ true }}" }
    });
    if let Some(name) = second_name {
        second["name"] = json!(name);
    }
    json!({
        "connectors": [first, second],
        "edges": [
            { "from": "t1", "to": "c1", "branch": null },
            { "from": "c1", "to": "c2", "branch": "Then" }
        ],
        "triggers": [{ "id": "t1", "kind": "Manual", "name": "Start", "description": "By hand" }]
    })
}

async fn save(app: &harness::App, workflow_id: &str, graph: Value) -> reqwest::Response {
    client()
        .put(app.workflow_versions_url(workflow_id))
        .bearer_auth(&app.token)
        .json(&json!({ "graph": graph, "layout": null }))
        .send()
        .await
        .expect("the api answers the save version call")
}

async fn current_graph(app: &harness::App, workflow_id: &str) -> Value {
    let read: Value = client()
        .get(app.workflow_url(workflow_id))
        .bearer_auth(&app.token)
        .send()
        .await
        .expect("the api answers the read")
        .json()
        .await
        .expect("json");
    read["data"]["current_version"]["graph"].clone()
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn node_names_descriptions_and_the_disabled_flag_survive_a_save_and_a_read() {
    let app = harness::start().await;
    let workflow_id = new_workflow(&app).await;

    let response = save(
        &app,
        &workflow_id,
        graph(
            json!({ "name": "Is it paid", "description": "Stops unpaid quotes", "disabled": true }),
            Some("Then notify"),
        ),
    )
    .await;
    assert_eq!(response.status(), 201);

    let stored = current_graph(&app, &workflow_id).await;
    assert_eq!(stored["connectors"][0]["name"], "Is it paid");
    assert_eq!(
        stored["connectors"][0]["description"],
        "Stops unpaid quotes"
    );
    assert_eq!(stored["connectors"][0]["disabled"], true);
    assert_eq!(stored["connectors"][1]["name"], "Then notify");
    assert_eq!(stored["connectors"][1]["disabled"], false);
    assert_eq!(stored["triggers"][0]["name"], "Start");
    assert_eq!(stored["triggers"][0]["description"], "By hand");

    app.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn a_payload_without_the_new_node_fields_is_still_accepted_and_reads_back_with_defaults() {
    let app = harness::start().await;
    let workflow_id = new_workflow(&app).await;
    let mut legacy = graph(json!({}), None);
    legacy["triggers"] = json!([{ "id": "t1", "kind": "Manual" }]);

    let response = save(&app, &workflow_id, legacy).await;
    assert_eq!(response.status(), 201);

    let stored = current_graph(&app, &workflow_id).await;
    assert_eq!(stored["connectors"][0]["name"], Value::Null);
    assert_eq!(stored["connectors"][0]["description"], Value::Null);
    assert_eq!(stored["connectors"][0]["disabled"], false);
    assert_eq!(stored["triggers"][0]["name"], Value::Null);
    assert_eq!(stored["triggers"][0]["description"], Value::Null);

    app.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn a_graph_stored_before_the_node_fields_existed_is_served_with_defaults() {
    let app = harness::start().await;
    let workflow_id = new_workflow(&app).await;
    let version_id = uuid::Uuid::now_v7();
    let legacy = json!({
        "connectors": [{
            "id": "c1",
            "kind": "flow.condition",
            "version": 1,
            "credential_id": null,
            "config": { "predicate": "{{ true }}" }
        }],
        "edges": [{ "from": "t1", "to": "c1", "branch": null }],
        "triggers": [{ "id": "t1", "kind": "Manual" }]
    });
    let workflow_uuid: uuid::Uuid = workflow_id.parse().expect("a uuid");
    sqlx::query("INSERT INTO automation.workflow_version (id, workflow_id, version, graph) VALUES ($1, $2, 1, $3)")
        .bind(version_id)
        .bind(workflow_uuid)
        .bind(&legacy)
        .execute(&app.pool)
        .await
        .expect("insert the legacy version");
    sqlx::query("UPDATE automation.workflow SET current_version_id = $2 WHERE id = $1")
        .bind(workflow_uuid)
        .bind(version_id)
        .execute(&app.pool)
        .await
        .expect("point at the legacy version");

    let stored = current_graph(&app, &workflow_id).await;

    assert_eq!(stored["connectors"][0]["id"], "c1");
    assert_eq!(stored["connectors"][0]["name"], Value::Null);
    assert_eq!(stored["connectors"][0]["disabled"], false);
    assert_eq!(stored["triggers"][0]["name"], Value::Null);

    app.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn invalid_node_names_and_descriptions_are_refused_with_the_node_and_field_named() {
    let app = harness::start().await;
    let workflow_id = new_workflow(&app).await;

    for (extra, second_name, field) in [
        (json!({ "name": "Same" }), Some("  same "), "name"),
        (json!({ "name": "   " }), None, "name"),
        (json!({ "name": "x".repeat(121) }), None, "name"),
        (
            json!({ "description": "x".repeat(501) }),
            None,
            "description",
        ),
    ] {
        let response = save(&app, &workflow_id, graph(extra.clone(), second_name)).await;
        assert_eq!(response.status(), 422, "{extra} / {second_name:?}");
        let body: Value = response.json().await.expect("json");
        assert_eq!(body["code"], "E_GRAPH_INVALID");
        let errors = body["details"]["errors"].as_array().expect("errors");
        let fault = errors
            .iter()
            .find(|error| error["field"] == field)
            .unwrap_or_else(|| panic!("no error on `{field}`: {errors:?}"));
        assert!(
            fault["connector_id"].is_string() || fault["trigger_id"].is_string(),
            "{fault}"
        );
        assert!(fault["message"].is_string());
    }

    app.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn a_trigger_and_a_connector_cannot_share_a_name() {
    let app = harness::start().await;
    let workflow_id = new_workflow(&app).await;

    let response = save(&app, &workflow_id, graph(json!({ "name": "start" }), None)).await;

    assert_eq!(response.status(), 422);
    let body: Value = response.json().await.expect("json");
    let errors = body["details"]["errors"].as_array().expect("errors");
    assert!(
        errors
            .iter()
            .any(|error| error["trigger_id"] == "t1" && error["field"] == "name"),
        "{errors:?}"
    );

    app.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn every_connector_descriptor_declares_how_it_runs_over_items() {
    let app = harness::start().await;

    let read: Value = client()
        .get(app.connectors_url())
        .bearer_auth(&app.token)
        .send()
        .await
        .expect("the api answers the catalogue call")
        .json()
        .await
        .expect("json");

    let connectors = read["data"]["connectors"].as_array().expect("connectors");
    assert!(!connectors.is_empty());
    for connector in connectors {
        let mode = connector["execution"].as_str().expect("an execution mode");
        assert!(["per_item", "once"].contains(&mode), "{connector}");
    }
    let execution_of = |kind: &str| {
        connectors
            .iter()
            .find(|connector| connector["kind"] == kind)
            .unwrap_or_else(|| panic!("`{kind}` is in the catalogue"))["execution"]
            .clone()
    };
    for once in [
        "flow.loop",
        "flow.merge",
        "flow.wait",
        "flow.config",
        "mestier.quote.search",
    ] {
        assert_eq!(execution_of(once), "once", "{once}");
    }
    for per_item in [
        "flow.condition",
        "mestier.customer.create",
        "http.request",
        "smtp.send",
    ] {
        assert_eq!(execution_of(per_item), "per_item", "{per_item}");
    }

    app.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn a_graph_reading_another_node_by_name_saves_and_the_old_syntax_is_refused() {
    let app = harness::start().await;
    let workflow_id = new_workflow(&app).await;
    let by_name = {
        let mut graph = graph(json!({ "name": "Premier" }), Some("Second"));
        graph["connectors"][1]["config"] =
            json!({ "predicate": "{{ $('Premier').item.json.matched }}" });
        graph
    };
    let by_id = {
        let mut graph = graph(json!({ "name": "Premier" }), Some("Second"));
        graph["connectors"][1]["config"] =
            json!({ "predicate": "{{ connectors.c1.output.matched }}" });
        graph
    };
    let unknown = {
        let mut graph = graph(json!({ "name": "Premier" }), Some("Second"));
        graph["connectors"][1]["config"] =
            json!({ "predicate": "{{ $('Fantôme').item.json.matched }}" });
        graph
    };

    let saved = save(&app, &workflow_id, by_name).await;
    let refused_old = save(&app, &workflow_id, by_id).await;
    let refused_unknown = save(&app, &workflow_id, unknown).await;

    assert_eq!(saved.status(), 201);
    assert_eq!(refused_old.status(), 422);
    let old: Value = refused_old.json().await.expect("json");
    let errors = old["details"]["errors"].as_array().expect("errors");
    let fault = errors
        .iter()
        .find(|error| error["connector_id"] == "c2" && error["field"] == "predicate")
        .unwrap_or_else(|| panic!("no error on c2.predicate: {errors:?}"));
    assert!(
        fault["message"]
            .as_str()
            .expect("a message")
            .contains("$('Nom du nœud').item.json"),
        "{fault}"
    );
    assert_eq!(refused_unknown.status(), 422);
    let unknown: Value = refused_unknown.json().await.expect("json");
    let errors = unknown["details"]["errors"].as_array().expect("errors");
    assert!(
        errors.iter().any(|error| error["connector_id"] == "c2"
            && error["field"] == "predicate"
            && error["message"]
                .as_str()
                .is_some_and(|m| m.contains("Fantôme"))),
        "{errors:?}"
    );

    app.cleanup().await;
}
