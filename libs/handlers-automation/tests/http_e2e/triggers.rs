use serde_json::{Value, json};
use uuid::Uuid;

use crate::{client, harness};

async fn new_workflow(app: &harness::App) -> String {
    let created: Value = client()
        .post(app.workflows_url())
        .bearer_auth(&app.token)
        .json(&json!({ "name": "Triggers" }))
        .send()
        .await
        .expect("the api answers the create call")
        .json()
        .await
        .expect("json");
    created["data"]["id"].as_str().expect("an id").to_owned()
}

fn condition() -> Value {
    json!({
        "id": "c1",
        "kind": "flow.condition",
        "version": 1,
        "credential_id": null,
        "config": { "predicate": "{{ true }}" }
    })
}

pub fn graph_of(triggers: Vec<(&str, Value)>) -> Value {
    json!({
        "connectors": [condition()],
        "edges": triggers
            .iter()
            .map(|(id, _)| json!({ "from": id, "to": "c1", "branch": null }))
            .collect::<Vec<_>>(),
        "triggers": triggers
            .into_iter()
            .map(|(id, kind)| json!({ "id": id, "kind": kind }))
            .collect::<Vec<_>>()
    })
}

pub fn webhook(path: &str) -> Value {
    json!({
        "Webhook": {
            "method": "POST",
            "path": path,
            "credential_id": null,
            "contract": [],
            "on_invalid": "Reject"
        }
    })
}

pub fn unique_path(label: &str) -> String {
    format!("{label}-{}", Uuid::now_v7().simple())
}

pub async fn save(app: &harness::App, workflow_id: &str, graph: Value) -> reqwest::Response {
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
async fn a_webhook_and_a_schedule_survive_a_save_and_a_read() {
    let app = harness::start().await;
    let workflow_id = new_workflow(&app).await;
    let webhook_kind = json!({
        "Webhook": {
            "method": "PUT",
            "path": unique_path("paiement/recu"),
            "credential_id": null,
            "contract": [
                { "name": "montant", "kind": "Number", "required": true, "example": "2340" },
                { "name": "client", "kind": "Object", "required": false, "example": "" }
            ],
            "on_invalid": "Accept"
        }
    });
    let schedule_kind = json!({
        "Schedule": {
            "frequency": "Weekly",
            "time": "09:00",
            "weekday": "Monday",
            "timezone": "Europe/Paris"
        }
    });

    let response = save(
        &app,
        &workflow_id,
        graph_of(vec![
            ("t1", webhook_kind.clone()),
            ("t2", schedule_kind.clone()),
        ]),
    )
    .await;
    assert_eq!(response.status(), 201);

    let stored = current_graph(&app, &workflow_id).await;
    assert_eq!(stored["triggers"][0]["kind"], webhook_kind);
    assert_eq!(stored["triggers"][1]["kind"], schedule_kind);

    app.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn optional_fields_may_be_left_out_of_the_new_triggers() {
    let app = harness::start().await;
    let workflow_id = new_workflow(&app).await;
    let path = unique_path("bare");

    let response = save(
        &app,
        &workflow_id,
        graph_of(vec![
            (
                "t1",
                json!({ "Webhook": { "method": "GET", "path": path } }),
            ),
            (
                "t2",
                json!({ "Schedule": { "frequency": "Daily", "time": "18:30" } }),
            ),
        ]),
    )
    .await;
    assert_eq!(response.status(), 201);

    let stored = current_graph(&app, &workflow_id).await;
    assert_eq!(
        stored["triggers"][0]["kind"]["Webhook"]["on_invalid"],
        "Reject"
    );
    assert_eq!(
        stored["triggers"][0]["kind"]["Webhook"]["contract"],
        json!([])
    );
    assert_eq!(
        stored["triggers"][1]["kind"]["Schedule"]["timezone"],
        Value::Null
    );

    app.cleanup().await;
}

async fn refused_fields(
    app: &harness::App,
    workflow_id: &str,
    kind: Value,
) -> Vec<(Option<String>, Option<String>)> {
    let response = save(app, workflow_id, graph_of(vec![("t1", kind)])).await;
    assert_eq!(response.status(), 422);
    let body: Value = response.json().await.expect("json");
    assert_eq!(body["code"], "E_GRAPH_INVALID");
    body["details"]["errors"]
        .as_array()
        .expect("errors")
        .iter()
        .map(|error| {
            assert_eq!(error["connector_id"], Value::Null);
            (
                error["trigger_id"].as_str().map(str::to_owned),
                error["field"].as_str().map(str::to_owned),
            )
        })
        .collect()
}

fn at(trigger: &str, field: &str) -> (Option<String>, Option<String>) {
    (Some(trigger.to_owned()), Some(field.to_owned()))
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn each_fault_of_a_trigger_comes_back_on_its_own_field() {
    let app = harness::start().await;
    let workflow_id = new_workflow(&app).await;

    assert_eq!(
        refused_fields(&app, &workflow_id, webhook("Bad Path")).await,
        vec![at("t1", "path")]
    );
    assert_eq!(
        refused_fields(&app, &workflow_id, webhook("webhooks/x")).await,
        vec![at("t1", "path")]
    );
    assert_eq!(
        refused_fields(&app, &workflow_id, webhook("a/../b")).await,
        vec![at("t1", "path")]
    );
    assert_eq!(
        refused_fields(
            &app,
            &workflow_id,
            json!({ "Webhook": {
                "method": "POST",
                "path": unique_path("credential"),
                "credential_id": Uuid::now_v7(),
                "contract": []
            } })
        )
        .await,
        vec![at("t1", "credential")]
    );
    assert_eq!(
        refused_fields(
            &app,
            &workflow_id,
            json!({ "Webhook": {
                "method": "POST",
                "path": unique_path("contract"),
                "contract": [
                    { "name": "montant", "kind": "Number", "required": true },
                    { "name": "montant", "kind": "Text", "required": false }
                ]
            } })
        )
        .await,
        vec![at("t1", "contract.montant")]
    );
    assert_eq!(
        refused_fields(
            &app,
            &workflow_id,
            json!({ "Schedule": { "frequency": "Daily", "time": "9h" } })
        )
        .await,
        vec![at("t1", "time")]
    );
    assert_eq!(
        refused_fields(
            &app,
            &workflow_id,
            json!({ "Webhook": {
                "method": "POST",
                "path": unique_path("keys"),
                "contract": [{ "name": "1 montant", "kind": "Number", "required": true }]
            } })
        )
        .await,
        vec![at("t1", "contract.1 montant")]
    );
    assert_eq!(
        refused_fields(
            &app,
            &workflow_id,
            json!({ "Schedule": { "frequency": "Daily", "time": "09:00", "timezone": "Mars/Olympus" } })
        )
        .await,
        vec![at("t1", "timezone")]
    );

    app.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn a_method_the_route_does_not_know_is_refused_at_the_door() {
    let app = harness::start().await;
    let workflow_id = new_workflow(&app).await;

    let response = save(
        &app,
        &workflow_id,
        graph_of(vec![(
            "t1",
            json!({ "Webhook": { "method": "TRACE", "path": unique_path("trace") } }),
        )]),
    )
    .await;

    assert!(response.status().is_client_error(), "{}", response.status());

    app.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn a_path_another_workflow_holds_is_a_409_naming_the_trigger() {
    let app = harness::start().await;
    let holder = new_workflow(&app).await;
    let candidate = new_workflow(&app).await;
    let path = unique_path("held");
    assert_eq!(
        save(&app, &holder, graph_of(vec![("t1", webhook(&path))]))
            .await
            .status(),
        201
    );

    let response = save(
        &app,
        &candidate,
        graph_of(vec![
            ("t7", webhook(&path)),
            ("t8", webhook(&unique_path("free"))),
        ]),
    )
    .await;

    assert_eq!(response.status(), 409);
    let body: Value = response.json().await.expect("json");
    assert_eq!(body["code"], "E_WEBHOOK_PATH_TAKEN");
    let errors = body["details"]["errors"].as_array().expect("errors");
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0]["trigger_id"], "t7");
    assert_eq!(errors[0]["field"], "path");
    assert_eq!(current_graph(&app, &candidate).await, Value::Null);

    app.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn a_workflow_can_save_its_own_path_again_and_free_it_by_dropping_the_trigger() {
    let app = harness::start().await;
    let first = new_workflow(&app).await;
    let second = new_workflow(&app).await;
    let path = unique_path("mine");

    for _ in 0..2 {
        assert_eq!(
            save(&app, &first, graph_of(vec![("t1", webhook(&path))]))
                .await
                .status(),
            201
        );
    }
    assert_eq!(
        save(&app, &second, graph_of(vec![("t1", webhook(&path))]))
            .await
            .status(),
        409
    );

    let manual = graph_of(vec![("t1", json!("Manual"))]);
    assert_eq!(save(&app, &first, manual).await.status(), 201);
    assert_eq!(
        save(&app, &second, graph_of(vec![("t1", webhook(&path))]))
            .await
            .status(),
        201
    );

    app.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn a_weekly_schedule_may_leave_its_weekday_out() {
    let app = harness::start().await;
    let workflow_id = new_workflow(&app).await;

    let response = save(
        &app,
        &workflow_id,
        graph_of(vec![(
            "t1",
            json!({ "Schedule": { "frequency": "Weekly", "time": "09:00" } }),
        )]),
    )
    .await;

    assert_eq!(response.status(), 201);
    let next: chrono::DateTime<chrono::Utc> = sqlx::query_scalar(
        "SELECT next_run_at FROM automation.schedule_state WHERE workflow_id = $1",
    )
    .bind(workflow_id.parse::<Uuid>().expect("a uuid"))
    .fetch_one(&app.pool)
    .await
    .expect("the schedule is registered");
    assert_eq!(chrono::Datelike::weekday(&next), chrono::Weekday::Mon);

    app.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn a_graph_holds_at_most_twenty_triggers() {
    let app = harness::start().await;
    let workflow_id = new_workflow(&app).await;
    let many = |count: usize| {
        let triggers: Vec<(String, Value)> = (0..count)
            .map(|index| (format!("t{index}"), json!("Manual")))
            .collect();
        graph_of(
            triggers
                .iter()
                .map(|(id, kind)| (id.as_str(), kind.clone()))
                .collect(),
        )
    };

    let over = save(&app, &workflow_id, many(21)).await;
    assert_eq!(over.status(), 422);
    let body: Value = over.json().await.expect("json");
    assert_eq!(body["details"]["errors"][0]["field"], "triggers");
    assert_eq!(body["details"]["errors"][0]["trigger_id"], Value::Null);

    assert_eq!(save(&app, &workflow_id, many(20)).await.status(), 201);

    app.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn an_organization_past_its_webhook_allowance_is_refused_with_a_static_message() {
    let app = harness::start().await;
    let holder = new_workflow(&app).await;
    for index in 0..199 {
        sqlx::query(
            "INSERT INTO automation.webhook_route (path, org_id, workflow_id, trigger_id, method) \
             VALUES ($1, $2, $3, $4, 'POST')",
        )
        .bind(unique_path(&format!("fill{index}")))
        .bind(app.organization_id)
        .bind(holder.parse::<Uuid>().expect("a uuid"))
        .bind(format!("fill-{index}"))
        .execute(&app.pool)
        .await
        .expect("fill the allowance");
    }
    let candidate = new_workflow(&app).await;

    let over = save(
        &app,
        &candidate,
        graph_of(vec![
            ("t1", webhook(&unique_path("over1"))),
            ("t2", webhook(&unique_path("over2"))),
        ]),
    )
    .await;
    let fits = save(
        &app,
        &candidate,
        graph_of(vec![("t1", webhook(&unique_path("fits")))]),
    )
    .await;

    assert_eq!(over.status(), 422);
    let body: Value = over.json().await.expect("json");
    assert_eq!(body["code"], "E_GRAPH_INVALID");
    assert_eq!(
        body["details"]["errors"][0]["message"],
        "an organization holds at most 200 webhook routes"
    );
    assert_eq!(fits.status(), 201);

    app.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn archiving_frees_the_path_and_re_enabling_needs_it_to_be_free_still() {
    let app = harness::start().await;
    let first = new_workflow(&app).await;
    let second = new_workflow(&app).await;
    let path = unique_path("archived");
    assert_eq!(
        save(&app, &first, graph_of(vec![("t1", webhook(&path))]))
            .await
            .status(),
        201
    );
    let patch = |workflow_id: &str, body: Value| {
        let request = client()
            .patch(app.workflow_url(workflow_id))
            .bearer_auth(&app.token)
            .json(&body);
        async move { request.send().await.expect("the api answers the update") }
    };

    assert!(
        patch(&first, json!({ "archived": true }))
            .await
            .status()
            .is_success()
    );
    assert_eq!(
        save(&app, &second, graph_of(vec![("t1", webhook(&path))]))
            .await
            .status(),
        201
    );
    assert!(
        patch(&first, json!({ "archived": false }))
            .await
            .status()
            .is_success()
    );
    let blocked = patch(&first, json!({ "enabled": true })).await;

    assert_eq!(blocked.status(), 409);
    let body: Value = blocked.json().await.expect("json");
    assert_eq!(body["code"], "E_WEBHOOK_PATH_TAKEN");

    app.cleanup().await;
}
