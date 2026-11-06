use serde_json::{Value, json};

use crate::{client, harness};

fn node(id: &str, kind: &str, config: Value) -> Value {
    json!({
        "id": id,
        "kind": kind,
        "version": 1,
        "credential_id": null,
        "config": config
    })
}

fn http_get(id: &str, url: &str) -> Value {
    node(id, "http.request", json!({ "method": "GET", "url": url }))
}

fn customer(id: &str, name: &str) -> Value {
    node(id, "mestier.customer.create", json!({ "name": name }))
}

fn edge(from: &str, to: &str, branch: Value) -> Value {
    json!({ "from": from, "to": to, "branch": branch })
}

fn draft(connectors: Vec<Value>, edges: Vec<Value>) -> Value {
    json!({
        "connectors": connectors,
        "edges": edges,
        "triggers": [{ "id": "t1", "kind": "Manual" }]
    })
}

fn linear_draft() -> Value {
    draft(
        vec![
            node("c1", "flow.config", json!({ "variables": { "n": 1 } })),
            http_get("c2", "https://example.com/a"),
            customer("c3", "Ada"),
        ],
        vec![
            edge("t1", "c1", Value::Null),
            edge("c1", "c2", Value::Null),
            edge("c2", "c3", Value::Null),
        ],
    )
}

async fn dry_run(app: &harness::App, token: &str, body: &Value) -> reqwest::Response {
    client()
        .post(app.dry_run_url())
        .bearer_auth(token)
        .json(body)
        .send()
        .await
        .expect("the api answers the dry run call")
}

async fn steps_of(response: reqwest::Response) -> Vec<Value> {
    assert_eq!(response.status(), 200);
    let envelope: Value = response.json().await.expect("the answer is json");
    envelope["data"]["steps"]
        .as_array()
        .expect("the answer carries steps")
        .clone()
}

fn step<'a>(steps: &'a [Value], id: &str) -> &'a Value {
    steps
        .iter()
        .find(|step| step["connector_id"] == id)
        .unwrap_or_else(|| panic!("no step for `{id}` in {steps:?}"))
}

fn ids(steps: &[Value]) -> Vec<&str> {
    steps
        .iter()
        .map(|step| step["connector_id"].as_str().expect("an id"))
        .collect()
}

async fn rows(app: &harness::App) -> (i64, i64, i64) {
    let count = async |statement: &str| -> i64 {
        sqlx::query_scalar(statement)
            .bind(app.organization_id)
            .fetch_one(&app.pool)
            .await
            .expect("count rows")
    };
    (
        count("SELECT COUNT(*) FROM automation.run WHERE org_id = $1").await,
        count(
            "SELECT COUNT(*) FROM automation.run_step s \
             JOIN automation.run r ON r.id = s.run_id WHERE r.org_id = $1",
        )
        .await,
        count("SELECT COUNT(*) FROM customers WHERE org_id = $1").await,
    )
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn a_member_without_manage_automation_cannot_dry_run() {
    let app = harness::start().await;
    let body = json!({ "graph": linear_draft() });

    for (who, token) in [
        ("no role", &app.no_role_token),
        ("view only", &app.view_only_token),
        ("outsider", &app.outsider_token),
    ] {
        let response = dry_run(&app, token, &body).await;
        assert_eq!(response.status(), 403, "{who}");
    }
    let allowed = dry_run(&app, &app.token, &body).await;
    assert_eq!(allowed.status(), 200);

    app.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn an_unsaved_draft_answers_one_step_per_reachable_node_and_writes_no_row() {
    let app = harness::start().await;
    let before = rows(&app).await;

    let response = dry_run(&app, &app.token, &json!({ "graph": linear_draft() })).await;
    let steps = steps_of(response).await;

    assert_eq!(ids(&steps), ["c1", "c2", "c3"]);
    for each in &steps {
        let mut keys: Vec<&str> = each
            .as_object()
            .expect("a step is an object")
            .keys()
            .map(String::as_str)
            .collect();
        keys.sort_unstable();
        assert_eq!(keys, ["connector_id", "error", "input", "output", "status"]);
        assert_eq!(each["status"], "succeeded");
        assert_eq!(each["error"], Value::Null);
    }
    assert_eq!(step(&steps, "c1")["output"], json!({ "n": 1 }));
    assert_eq!(
        step(&steps, "c1")["input"],
        json!({ "variables": { "n": 1 } })
    );
    assert_eq!(
        step(&steps, "c2")["output"],
        json!({ "status": 200, "headers": {}, "body": {} })
    );
    assert_eq!(
        step(&steps, "c2")["input"],
        json!({ "method": "GET", "url": "https://example.com/a" })
    );
    assert_eq!(rows(&app).await, before, "no run, no step, no customer");

    app.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn an_invalid_node_answers_422_naming_the_node_and_the_field() {
    let app = harness::start().await;
    let graph = draft(
        vec![node("c1", "flow.condition", json!({}))],
        vec![edge("t1", "c1", Value::Null)],
    );

    let response = dry_run(&app, &app.token, &json!({ "graph": graph })).await;

    assert_eq!(response.status(), 422);
    let body: Value = response.json().await.expect("json");
    assert_eq!(body["code"], "E_GRAPH_INVALID");
    assert_eq!(body["status"], 422);
    let errors = body["details"]["errors"].as_array().expect("errors");
    let fault = errors
        .iter()
        .find(|error| error["connector_id"] == "c1" && error["field"] == "predicate")
        .unwrap_or_else(|| panic!("no error on c1.predicate: {errors:?}"));
    assert!(
        fault["message"]
            .as_str()
            .expect("a message")
            .contains("predicate")
    );
    assert_eq!(rows(&app).await, (0, 0, 0));

    app.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn a_condition_follows_the_branch_its_sample_data_selects() {
    let app = harness::start().await;
    let graph = draft(
        vec![
            http_get("c1", "https://example.com/a"),
            node(
                "c2",
                "flow.condition",
                json!({ "predicate": "{{ connectors.c1.output.status == 200 }}" }),
            ),
            customer("c3", "then"),
            customer("c4", "else"),
        ],
        vec![
            edge("t1", "c1", Value::Null),
            edge("c1", "c2", Value::Null),
            edge("c2", "c3", json!("Then")),
            edge("c2", "c4", json!("Else")),
        ],
    );

    let steps = steps_of(dry_run(&app, &app.token, &json!({ "graph": graph })).await).await;

    assert_eq!(step(&steps, "c2")["output"], json!({ "matched": true }));
    assert_eq!(step(&steps, "c3")["status"], "succeeded");
    assert_eq!(step(&steps, "c4")["status"], "skipped");
    assert_eq!(step(&steps, "c4")["input"], Value::Null);
    assert_eq!(step(&steps, "c4")["output"], Value::Null);

    app.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn a_loop_iterates_at_most_three_sample_items() {
    let app = harness::start().await;
    let graph = draft(
        vec![
            node("c1", "flow.loop", json!({ "items": "{{ trigger.items }}" })),
            customer("c2", "{{ loop.item }}"),
            customer("c3", "done"),
        ],
        vec![
            edge("t1", "c1", Value::Null),
            edge("c1", "c2", json!("Each")),
            edge("c1", "c3", json!("After")),
        ],
    );
    let body = json!({
        "graph": graph,
        "trigger_payload": { "items": ["a", "b", "c", "d", "e"] }
    });

    let steps = steps_of(dry_run(&app, &app.token, &body).await).await;

    assert_eq!(ids(&steps), ["c1", "c2", "c2", "c2", "c3"]);
    let names: Vec<&Value> = steps
        .iter()
        .filter(|step| step["connector_id"] == "c2")
        .map(|step| &step["input"]["name"])
        .collect();
    assert_eq!(names, [&json!("a"), &json!("b"), &json!("c")]);

    app.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn a_disabled_node_passes_the_previous_output_through() {
    let app = harness::start().await;
    let mut disabled = customer("c2", "never created");
    disabled["disabled"] = json!(true);
    let graph = draft(
        vec![
            node("c1", "flow.config", json!({ "variables": { "n": 1 } })),
            disabled,
        ],
        vec![edge("t1", "c1", Value::Null), edge("c1", "c2", Value::Null)],
    );
    let before = rows(&app).await;

    let steps = steps_of(dry_run(&app, &app.token, &json!({ "graph": graph })).await).await;

    assert_eq!(step(&steps, "c2")["status"], "succeeded");
    assert_eq!(step(&steps, "c2")["output"], json!({ "n": 1 }));
    assert_eq!(step(&steps, "c2")["input"], Value::Null);
    assert_eq!(rows(&app).await, before);

    app.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn a_reference_to_an_output_that_does_not_exist_fails_the_referencing_node() {
    let app = harness::start().await;
    let graph = draft(
        vec![
            http_get("c1", "https://example.com/a"),
            http_get(
                "c2",
                "https://example.com/{{ connectors.c1.output.body.nope }}",
            ),
            customer("c3", "never reached"),
        ],
        vec![
            edge("t1", "c1", Value::Null),
            edge("c1", "c2", Value::Null),
            edge("c2", "c3", Value::Null),
        ],
    );

    let steps = steps_of(dry_run(&app, &app.token, &json!({ "graph": graph })).await).await;

    assert_eq!(ids(&steps), ["c1", "c2"]);
    assert_eq!(step(&steps, "c2")["status"], "failed");
    assert!(
        step(&steps, "c2")["error"]
            .as_str()
            .expect("an error")
            .contains("connectors.c1.output.body.nope")
    );
    assert_eq!(step(&steps, "c1")["status"], "succeeded");

    app.cleanup().await;
}

fn collection_body() -> Value {
    json!({
        "title": "Addresses",
        "version": "1.0.0",
        "description": "",
        "base_url": "https://api.example.com/v1",
        "source": { "kind": "url", "url": "https://api.example.com/openapi.json" },
        "security": [],
        "operations": [{
            "id": "getAddress",
            "method": "GET",
            "path": "/addresses/{id}",
            "summary": "",
            "tag": "",
            "parameters": [{
                "name": "id",
                "location": "path",
                "required": true,
                "description": "",
                "schema": { "type": "string" }
            }],
            "request_body": null,
            "response": { "city": "Lyon" },
            "response_status": "201"
        }],
        "schemas": {}
    })
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn a_collection_operation_answers_with_its_stored_example_until_the_collection_is_deleted() {
    let app = harness::start().await;
    let imported: Value = client()
        .post(app.collections_url())
        .bearer_auth(&app.token)
        .json(&collection_body())
        .send()
        .await
        .expect("the api answers the import")
        .json()
        .await
        .expect("json");
    let collection_id = imported["data"]["id"].as_str().expect("an id").to_owned();
    let mut call = node(
        "c1",
        "collection.operation",
        json!({ "path": { "id": "1" } }),
    );
    call["operation"] = json!({ "collection_id": collection_id, "operation_id": "getAddress" });
    let graph = draft(vec![call], vec![edge("t1", "c1", Value::Null)]);
    let body = json!({ "graph": graph });

    let steps = steps_of(dry_run(&app, &app.token, &body).await).await;
    let deleted = client()
        .delete(app.collection_url(&collection_id))
        .bearer_auth(&app.token)
        .send()
        .await
        .expect("the api answers the delete");
    assert_eq!(deleted.status(), 204);
    let refused = dry_run(&app, &app.token, &body).await;

    assert_eq!(
        step(&steps, "c1")["output"],
        json!({ "status": 201, "headers": {}, "body": { "city": "Lyon" } })
    );
    assert_eq!(refused.status(), 422);
    let refusal: Value = refused.json().await.expect("json");
    assert_eq!(refusal["code"], "E_GRAPH_INVALID");
    let errors = refusal["details"]["errors"].as_array().expect("errors");
    assert!(
        errors.iter().any(|error| error["connector_id"] == "c1"
            && error["message"]
                .as_str()
                .is_some_and(|message| message.contains(&collection_id))),
        "{errors:?}"
    );

    app.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn an_unknown_trigger_and_a_payload_that_is_not_an_object_are_refused() {
    let app = harness::start().await;

    let unknown = dry_run(
        &app,
        &app.token,
        &json!({ "graph": linear_draft(), "trigger_id": "ghost" }),
    )
    .await;
    let scalar = dry_run(
        &app,
        &app.token,
        &json!({ "graph": linear_draft(), "trigger_payload": 42 }),
    )
    .await;
    let selected = dry_run(
        &app,
        &app.token,
        &json!({ "graph": linear_draft(), "trigger_id": "t1", "trigger_payload": {} }),
    )
    .await;

    assert_eq!(unknown.status(), 422);
    let refusal: Value = unknown.json().await.expect("json");
    assert!(
        refusal["message"]
            .as_str()
            .expect("a message")
            .contains("ghost")
    );
    assert_eq!(scalar.status(), 422);
    assert_eq!(selected.status(), 200);

    app.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn a_body_over_the_limit_is_refused_exactly_as_a_save_refuses_it() {
    let app = harness::start().await;
    let created: Value = client()
        .post(app.workflows_url())
        .bearer_auth(&app.token)
        .json(&json!({ "name": "Oversized" }))
        .send()
        .await
        .expect("create")
        .json()
        .await
        .expect("json");
    let workflow_id = created["data"]["id"].as_str().expect("an id").to_owned();
    let mut graph = linear_draft();
    graph["connectors"][0]["config"]["variables"] = json!({ "blob": "x".repeat(3 * 1024 * 1024) });

    let dry = dry_run(&app, &app.token, &json!({ "graph": graph.clone() })).await;
    let save = client()
        .put(app.workflow_versions_url(&workflow_id))
        .bearer_auth(&app.token)
        .json(&json!({ "graph": graph, "layout": null }))
        .send()
        .await
        .expect("the api answers the save");

    assert_eq!(dry.status(), 413);
    assert_eq!(dry.status(), save.status());

    app.cleanup().await;
}
