use serde_json::{Value, json};

use crate::{client, harness};

fn collection_body(title: &str, version: &str) -> Value {
    json!({
        "title": title,
        "version": version,
        "description": "Addresses",
        "base_url": "https://api.example.com/v1",
        "source": { "kind": "url", "url": "https://api.example.com/openapi.json" },
        "security": [
            { "name": "bearerAuth", "kind": { "type": "http", "scheme": "bearer" } }
        ],
        "operations": [
            {
                "id": "getAddress",
                "method": "GET",
                "path": "/addresses/{id}",
                "summary": "Read one",
                "tag": "Addresses",
                "parameters": [
                    {
                        "name": "id",
                        "location": "path",
                        "required": true,
                        "description": "",
                        "schema": { "type": "string" }
                    }
                ],
                "request_body": null,
                "response": { "type": "object" },
                "response_status": "200"
            },
            {
                "id": "createAddress",
                "method": "POST",
                "path": "/addresses",
                "summary": "Create",
                "tag": "Addresses",
                "parameters": [],
                "request_body": { "schema": { "type": "object" } },
                "response": null,
                "response_status": "201"
            }
        ],
        "schemas": { "Address": { "type": "object" } }
    })
}

async fn post_collection(app: &harness::App, token: &str, body: &Value) -> reqwest::Response {
    client()
        .post(app.collections_url())
        .bearer_auth(token)
        .json(body)
        .send()
        .await
        .expect("the api answers the collection import call")
}

async fn imported(app: &harness::App, title: &str, version: &str) -> Value {
    let response = post_collection(app, &app.token, &collection_body(title, version)).await;
    assert_eq!(response.status(), 201);
    let envelope: Value = response.json().await.expect("the answer is json");
    envelope["data"].clone()
}

fn id_of(collection: &Value) -> &str {
    collection["id"]
        .as_str()
        .expect("a collection carries an id")
}

async fn body_of(response: reqwest::Response) -> (u16, Value) {
    let status = response.status().as_u16();
    (status, response.json().await.expect("the answer is json"))
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn an_imported_collection_has_the_contracted_shape() {
    let app = harness::start().await;

    let collection = imported(&app, "API Adresse", "1.0.0").await;

    let mut keys: Vec<&str> = collection
        .as_object()
        .expect("a collection is an object")
        .keys()
        .map(String::as_str)
        .collect();
    keys.sort_unstable();
    assert_eq!(
        keys,
        [
            "base_url",
            "description",
            "history",
            "id",
            "imported_at",
            "operations",
            "organization_id",
            "schemas",
            "security",
            "slug",
            "source",
            "title",
            "version"
        ]
    );
    assert_eq!(collection["slug"], "api-adresse");
    assert_eq!(collection["organization_id"], json!(app.organization_id));
    assert_eq!(collection["history"], json!([]));
    assert_eq!(collection["operations"][0]["method"], "GET");
    assert_eq!(
        collection["operations"][0]["parameters"][0]["location"],
        "path"
    );
    assert_eq!(collection["security"][0]["kind"]["scheme"], "bearer");

    let (status, read) = body_of(
        client()
            .get(app.collection_url(id_of(&collection)))
            .bearer_auth(&app.token)
            .send()
            .await
            .expect("the api answers the collection read call"),
    )
    .await;
    assert_eq!(status, 200);
    assert_eq!(read["data"], collection);

    app.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn a_clashing_title_gets_a_suffixed_slug() {
    let app = harness::start().await;

    let first = imported(&app, "API Adresse", "1.0.0").await;
    let second = imported(&app, "api adresse", "1.0.0").await;
    let third = imported(&app, "API Adresse", "2.0.0").await;

    assert_eq!(first["slug"], "api-adresse");
    assert_eq!(second["slug"], "api-adresse-2");
    assert_eq!(third["slug"], "api-adresse-3");

    app.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn the_list_is_light_and_carries_the_operation_count() {
    let app = harness::start().await;
    imported(&app, "Beta", "1.0.0").await;
    imported(&app, "alpha", "1.0.0").await;

    let (status, listed) = body_of(
        client()
            .get(app.collections_url())
            .bearer_auth(&app.token)
            .send()
            .await
            .expect("the api answers the collection list call"),
    )
    .await;

    assert_eq!(status, 200);
    let items = listed["data"].as_array().expect("a list");
    let titles: Vec<&str> = items
        .iter()
        .map(|item| item["title"].as_str().unwrap())
        .collect();
    assert_eq!(titles, ["alpha", "Beta"]);
    for item in items {
        let object = item.as_object().unwrap();
        assert!(!object.contains_key("operations"));
        assert!(!object.contains_key("schemas"));
        assert!(!object.contains_key("history"));
        assert!(!object.contains_key("description"));
        assert_eq!(item["operation_count"], 2);
        assert!(object.contains_key("security"));
    }

    app.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn another_organization_does_not_see_a_collection() {
    let app = harness::start().await;
    let collection = imported(&app, "Private", "1.0.0").await;

    let (status, elsewhere) = body_of(
        client()
            .get(app.other_organization_collections_url())
            .bearer_auth(&app.outsider_token)
            .send()
            .await
            .expect("the api answers the outsider's list call"),
    )
    .await;
    assert_eq!(status, 200);
    assert_eq!(elsewhere["data"], json!([]));

    let read_through_the_other_door = client()
        .get(format!(
            "{}/{}",
            app.other_organization_collections_url(),
            id_of(&collection)
        ))
        .bearer_auth(&app.outsider_token)
        .send()
        .await
        .expect("the api answers the cross-organization read");
    assert_eq!(read_through_the_other_door.status(), 404);

    let replace_through_the_other_door = client()
        .put(format!(
            "{}/{}",
            app.other_organization_collections_url(),
            id_of(&collection)
        ))
        .bearer_auth(&app.outsider_token)
        .json(&collection_body("Hijacked", "9.9.9"))
        .send()
        .await
        .expect("the api answers the cross-organization replace");
    assert_eq!(replace_through_the_other_door.status(), 404);

    let delete_through_the_other_door = client()
        .delete(format!(
            "{}/{}",
            app.other_organization_collections_url(),
            id_of(&collection)
        ))
        .bearer_auth(&app.outsider_token)
        .send()
        .await
        .expect("the api answers the cross-organization delete");
    assert_eq!(delete_through_the_other_door.status(), 404);

    let outsider_reading_mine = client()
        .get(app.collections_url())
        .bearer_auth(&app.outsider_token)
        .send()
        .await
        .expect("the api answers the outsider's call on my organization");
    assert_eq!(outsider_reading_mine.status(), 403);

    let still_there = client()
        .get(app.collection_url(id_of(&collection)))
        .bearer_auth(&app.token)
        .send()
        .await
        .expect("the api answers the owner's read");
    assert_eq!(still_there.status(), 200);

    app.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn replacing_keeps_the_previous_version_in_the_history() {
    let app = harness::start().await;
    let first = imported(&app, "Annuaire", "1.0.0").await;

    let (status, replaced) = body_of(
        client()
            .put(app.collection_url(id_of(&first)))
            .bearer_auth(&app.token)
            .json(&collection_body("Annuaire v2", "2.0.0"))
            .send()
            .await
            .expect("the api answers the collection replace call"),
    )
    .await;
    assert_eq!(status, 200);
    assert_eq!(replaced["data"]["id"], first["id"]);
    assert_eq!(replaced["data"]["slug"], "annuaire");
    assert_eq!(replaced["data"]["title"], "Annuaire v2");
    assert_eq!(replaced["data"]["version"], "2.0.0");
    assert_eq!(replaced["data"]["history"].as_array().unwrap().len(), 1);
    assert_eq!(replaced["data"]["history"][0]["version"], "1.0.0");

    let (_, read) = body_of(
        client()
            .get(app.collection_url(id_of(&first)))
            .bearer_auth(&app.token)
            .send()
            .await
            .expect("the api answers the collection read call"),
    )
    .await;
    let history = read["data"]["history"].as_array().expect("a history");
    assert_eq!(history.len(), 1);
    assert_eq!(history[0]["version"], "1.0.0");
    assert_eq!(history[0]["imported_at"], first["imported_at"]);
    assert_eq!(history[0]["source"], first["source"]);
    assert_eq!(history[0]["base_url"], first["base_url"]);
    assert_eq!(history[0]["security"], first["security"]);

    app.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn deleting_removes_the_collection_and_its_history() {
    let app = harness::start().await;
    let first = imported(&app, "Annuaire", "1.0.0").await;
    let replaced = client()
        .put(app.collection_url(id_of(&first)))
        .bearer_auth(&app.token)
        .json(&collection_body("Annuaire", "2.0.0"))
        .send()
        .await
        .expect("the api answers the collection replace call");
    assert_eq!(replaced.status(), 200);

    let deleted = client()
        .delete(app.collection_url(id_of(&first)))
        .bearer_auth(&app.token)
        .send()
        .await
        .expect("the api answers the collection delete call");
    assert_eq!(deleted.status(), 204);

    let read = client()
        .get(app.collection_url(id_of(&first)))
        .bearer_auth(&app.token)
        .send()
        .await
        .expect("the api answers the collection read call");
    assert_eq!(read.status(), 404);
    let (_, listed) = body_of(
        client()
            .get(app.collections_url())
            .bearer_auth(&app.token)
            .send()
            .await
            .expect("the api answers the collection list call"),
    )
    .await;
    assert_eq!(listed["data"], json!([]));
    let history_rows: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM automation.collection_version WHERE collection_id = $1::uuid",
    )
    .bind(id_of(&first))
    .fetch_one(&app.pool)
    .await
    .expect("count the history rows");
    assert_eq!(history_rows, 0);

    let again = client()
        .delete(app.collection_url(id_of(&first)))
        .bearer_auth(&app.token)
        .send()
        .await
        .expect("the api answers the second delete call");
    assert_eq!(again.status(), 404);

    app.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn an_unknown_collection_is_not_found_on_read_replace_and_delete() {
    let app = harness::start().await;
    let missing = app.collection_url("00000000-0000-7000-8000-000000000999");

    let read = client()
        .get(&missing)
        .bearer_auth(&app.token)
        .send()
        .await
        .unwrap();
    let replace = client()
        .put(&missing)
        .bearer_auth(&app.token)
        .json(&collection_body("Ghost", "1.0.0"))
        .send()
        .await
        .unwrap();
    let delete = client()
        .delete(&missing)
        .bearer_auth(&app.token)
        .send()
        .await
        .unwrap();

    assert_eq!(read.status(), 404);
    assert_eq!(replace.status(), 404);
    assert_eq!(delete.status(), 404);

    app.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn collections_follow_the_automation_permission_split() {
    let app = harness::start().await;
    let collection = imported(&app, "Gated", "1.0.0").await;
    let one = app.collection_url(id_of(&collection));

    for token in [&app.no_role_token] {
        let list = client()
            .get(app.collections_url())
            .bearer_auth(token)
            .send()
            .await
            .unwrap();
        let get = client().get(&one).bearer_auth(token).send().await.unwrap();
        let create = post_collection(&app, token, &collection_body("Nope", "1")).await;
        assert_eq!(list.status(), 403);
        assert_eq!(get.status(), 403);
        assert_eq!(create.status(), 403);
    }

    let token = &app.view_only_token;
    let list = client()
        .get(app.collections_url())
        .bearer_auth(token)
        .send()
        .await
        .unwrap();
    let get = client().get(&one).bearer_auth(token).send().await.unwrap();
    let create = post_collection(&app, token, &collection_body("Nope", "1")).await;
    let replace = client()
        .put(&one)
        .bearer_auth(token)
        .json(&collection_body("Nope", "2"))
        .send()
        .await
        .unwrap();
    let delete = client()
        .delete(&one)
        .bearer_auth(token)
        .send()
        .await
        .unwrap();
    assert_eq!(list.status(), 200);
    assert_eq!(get.status(), 200);
    assert_eq!(create.status(), 403);
    assert_eq!(replace.status(), 403);
    assert_eq!(delete.status(), 403);

    let untouched = client()
        .get(&one)
        .bearer_auth(&app.token)
        .send()
        .await
        .unwrap();
    assert_eq!(untouched.status(), 200);
    let (_, listed) = body_of(
        client()
            .get(app.collections_url())
            .bearer_auth(&app.token)
            .send()
            .await
            .unwrap(),
    )
    .await;
    assert_eq!(listed["data"].as_array().unwrap().len(), 1);

    app.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn a_spec_over_the_size_limit_is_rejected_and_stores_nothing() {
    let app = harness::start().await;
    let mut body = collection_body("Huge", "1.0.0");
    body["schemas"] = json!({ "Big": { "description": "x".repeat(2 * 1024 * 1024 + 4096) } });

    let (status, answer) = body_of(post_collection(&app, &app.token, &body).await).await;

    assert_eq!(status, 413);
    assert_eq!(answer["code"], "E_COLLECTION_TOO_LARGE");
    assert!(answer["message"].as_str().unwrap().contains("2097152"));

    let mut beyond_the_door = collection_body("Huger", "1.0.0");
    beyond_the_door["schemas"] = json!({ "Big": { "description": "x".repeat(3 * 1024 * 1024) } });
    let (status, answer) = body_of(post_collection(&app, &app.token, &beyond_the_door).await).await;
    assert_eq!(status, 413);
    assert_eq!(answer["code"], "E_COLLECTION_TOO_LARGE");
    assert_eq!(answer["status"], 413);
    assert!(answer["message"].as_str().unwrap().contains("2359296"));

    let replaced = client()
        .put(app.collection_url("00000000-0000-7000-8000-000000000999"))
        .bearer_auth(&app.token)
        .json(&beyond_the_door)
        .send()
        .await
        .expect("the api answers the oversized replace");
    assert_eq!(replaced.status(), 413);
    assert_eq!(
        replaced.json::<Value>().await.unwrap()["code"],
        "E_COLLECTION_TOO_LARGE"
    );

    let (_, listed) = body_of(
        client()
            .get(app.collections_url())
            .bearer_auth(&app.token)
            .send()
            .await
            .unwrap(),
    )
    .await;
    assert_eq!(listed["data"], json!([]));

    let mut near_the_limit = collection_body("Large but fine", "1.0.0");
    near_the_limit["schemas"] =
        json!({ "Big": { "description": "x".repeat(2 * 1024 * 1024 - 4096) } });
    let accepted = post_collection(&app, &app.token, &near_the_limit).await;
    assert_eq!(accepted.status(), 201);

    app.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn an_invalid_model_is_refused_with_the_reason() {
    let app = harness::start().await;

    let mut no_operation = collection_body("Empty", "1.0.0");
    no_operation["operations"] = json!([]);
    let (status, answer) = body_of(post_collection(&app, &app.token, &no_operation).await).await;
    assert_eq!(status, 400);
    assert_eq!(answer["code"], "E_VALIDATION");
    assert!(answer["message"].as_str().unwrap().contains("operation"));

    let mut blank_title = collection_body("  ", "1.0.0");
    blank_title["title"] = json!("  ");
    let (status, _) = body_of(post_collection(&app, &app.token, &blank_title).await).await;
    assert_eq!(status, 400);

    let mut duplicated = collection_body("Twice", "1.0.0");
    let first = duplicated["operations"][0].clone();
    duplicated["operations"].as_array_mut().unwrap().push(first);
    let (status, answer) = body_of(post_collection(&app, &app.token, &duplicated).await).await;
    assert_eq!(status, 400);
    assert!(answer["message"].as_str().unwrap().contains("getAddress"));

    let mut unknown_method = collection_body("Method", "1.0.0");
    unknown_method["operations"][0]["method"] = json!("TRACE");
    let rejected = post_collection(&app, &app.token, &unknown_method).await;
    assert_eq!(rejected.status(), 422);

    app.cleanup().await;
}

async fn new_workflow(app: &harness::App) -> String {
    let created: Value = client()
        .post(app.workflows_url())
        .bearer_auth(&app.token)
        .json(&json!({ "name": "Uses a collection" }))
        .send()
        .await
        .expect("the api answers the create call")
        .json()
        .await
        .expect("json");
    created["data"]["id"].as_str().expect("an id").to_owned()
}

fn operation_graph(collection_id: &str, operation_id: &str, config: Value) -> Value {
    json!({
        "connectors": [{
            "id": "c1",
            "kind": "collection.operation",
            "version": 1,
            "credential_id": null,
            "config": config,
            "operation": { "collection_id": collection_id, "operation_id": operation_id }
        }],
        "edges": [{ "from": "t1", "to": "c1", "branch": null }],
        "triggers": [{ "id": "t1", "kind": "Manual" }]
    })
}

async fn save_graph(app: &harness::App, workflow_id: &str, graph: Value) -> reqwest::Response {
    client()
        .put(app.workflow_versions_url(workflow_id))
        .bearer_auth(&app.token)
        .json(&json!({ "graph": graph, "layout": null }))
        .send()
        .await
        .expect("the api answers the save version call")
}

fn graph_errors(answer: &Value) -> Vec<(Option<String>, Option<String>, String)> {
    answer["details"]["errors"]
        .as_array()
        .expect("a graph refusal lists its errors")
        .iter()
        .map(|error| {
            (
                error["connector_id"].as_str().map(str::to_owned),
                error["field"].as_str().map(str::to_owned),
                error["message"].as_str().unwrap_or_default().to_owned(),
            )
        })
        .collect()
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn a_node_on_a_stored_operation_saves_and_reads_back_with_its_reference_only() {
    let app = harness::start().await;
    let collection = imported(&app, "Adresses", "1.0.0").await;
    let workflow_id = new_workflow(&app).await;
    let graph = operation_graph(
        id_of(&collection),
        "getAddress",
        json!({ "path": { "id": "42" } }),
    );

    let saved = save_graph(&app, &workflow_id, graph).await;
    assert_eq!(saved.status(), 201);

    let (status, read) = body_of(
        client()
            .get(app.workflow_url(&workflow_id))
            .bearer_auth(&app.token)
            .send()
            .await
            .expect("the api answers the workflow read call"),
    )
    .await;
    assert_eq!(status, 200);
    let node = &read["data"]["current_version"]["graph"]["connectors"][0];
    assert_eq!(node["kind"], "collection.operation");
    assert_eq!(node["operation"]["collection_id"], collection["id"]);
    assert_eq!(node["operation"]["operation_id"], "getAddress");
    assert_eq!(node["config"], json!({ "path": { "id": "42" } }));
    let serialized = node.to_string();
    assert!(!serialized.contains("api.example.com"), "{serialized}");
    assert!(!serialized.contains("\"url\""), "{serialized}");

    app.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn an_unknown_operation_a_missing_parameter_and_an_unknown_field_are_refused_naming_node_and_field()
 {
    let app = harness::start().await;
    let collection = imported(&app, "Adresses", "1.0.0").await;
    let workflow_id = new_workflow(&app).await;

    let unknown_operation = save_graph(
        &app,
        &workflow_id,
        operation_graph(id_of(&collection), "teleport", json!({})),
    )
    .await;
    let (status, answer) = body_of(unknown_operation).await;
    assert_eq!(status, 422);
    assert_eq!(answer["code"], "E_GRAPH_INVALID");
    let errors = graph_errors(&answer);
    assert_eq!(errors.len(), 1, "{errors:?}");
    assert_eq!(errors[0].0.as_deref(), Some("c1"));
    assert_eq!(errors[0].1.as_deref(), Some("operation"));
    assert!(errors[0].2.contains("teleport"), "{}", errors[0].2);

    let missing = save_graph(
        &app,
        &workflow_id,
        operation_graph(id_of(&collection), "getAddress", json!({})),
    )
    .await;
    let (status, answer) = body_of(missing).await;
    assert_eq!(status, 422);
    let errors = graph_errors(&answer);
    assert_eq!(errors.len(), 1, "{errors:?}");
    assert_eq!(errors[0].0.as_deref(), Some("c1"));
    assert_eq!(errors[0].1.as_deref(), Some("path.id"));

    let unknown_field = save_graph(
        &app,
        &workflow_id,
        operation_graph(
            id_of(&collection),
            "getAddress",
            json!({ "path": { "id": "1" }, "query": { "sort": "asc" }, "url": "http://copied" }),
        ),
    )
    .await;
    let (status, answer) = body_of(unknown_field).await;
    assert_eq!(status, 422);
    let fields: Vec<Option<String>> = graph_errors(&answer).into_iter().map(|e| e.1).collect();
    assert!(
        fields.contains(&Some("query.sort".to_owned())),
        "{fields:?}"
    );
    assert!(fields.contains(&Some("url".to_owned())), "{fields:?}");

    let current: Value = client()
        .get(app.workflow_url(&workflow_id))
        .bearer_auth(&app.token)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(current["data"]["current_version"], Value::Null);

    app.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn deleting_a_collection_does_not_touch_the_workflow_but_its_next_save_names_the_collection()
{
    let app = harness::start().await;
    let collection = imported(&app, "Adresses", "1.0.0").await;
    let workflow_id = new_workflow(&app).await;
    let graph = operation_graph(
        id_of(&collection),
        "getAddress",
        json!({ "path": { "id": "42" } }),
    );
    assert_eq!(
        save_graph(&app, &workflow_id, graph.clone()).await.status(),
        201
    );

    let deleted = client()
        .delete(app.collection_url(id_of(&collection)))
        .bearer_auth(&app.token)
        .send()
        .await
        .expect("the api answers the collection delete call");
    assert_eq!(deleted.status(), 204);

    let (status, answer) = body_of(save_graph(&app, &workflow_id, graph).await).await;
    assert_eq!(status, 422);
    let errors = graph_errors(&answer);
    assert_eq!(errors.len(), 1, "{errors:?}");
    assert_eq!(errors[0].0.as_deref(), Some("c1"));
    assert_eq!(errors[0].1.as_deref(), Some("operation"));
    assert!(errors[0].2.contains(id_of(&collection)), "{}", errors[0].2);

    let read = client()
        .get(app.workflow_url(&workflow_id))
        .bearer_auth(&app.token)
        .send()
        .await
        .unwrap();
    assert_eq!(read.status(), 200);

    app.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn a_collection_of_another_organization_cannot_back_a_node() {
    let app = harness::start().await;
    let foreign: Value = client()
        .post(app.other_organization_collections_url())
        .bearer_auth(&app.outsider_token)
        .json(&collection_body("Foreign", "1.0.0"))
        .send()
        .await
        .expect("the api answers the foreign import")
        .json()
        .await
        .unwrap();
    let foreign_id = foreign["data"]["id"].as_str().unwrap().to_owned();
    let workflow_id = new_workflow(&app).await;

    let (status, answer) = body_of(
        save_graph(
            &app,
            &workflow_id,
            operation_graph(&foreign_id, "getAddress", json!({ "path": { "id": "1" } })),
        )
        .await,
    )
    .await;

    assert_eq!(status, 422);
    let errors = graph_errors(&answer);
    assert_eq!(errors[0].1.as_deref(), Some("operation"));
    assert!(
        errors[0].2.contains("unknown collection"),
        "{}",
        errors[0].2
    );

    app.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn the_model_is_cleaned_or_refused_before_it_is_stored() {
    let app = harness::start().await;

    let mut with_credentials = collection_body("Credentialed", "1.0.0");
    with_credentials["base_url"] = json!("https://bot:secret@api.example.com/v1");
    let (status, answer) =
        body_of(post_collection(&app, &app.token, &with_credentials).await).await;
    assert_eq!(status, 400);
    assert!(answer["message"].as_str().unwrap().contains("base url"));

    let mut with_query = collection_body("Queried", "1.0.0");
    with_query["base_url"] = json!("https://api.example.com/v1?key=1");
    assert_eq!(
        post_collection(&app, &app.token, &with_query)
            .await
            .status(),
        400
    );

    let mut nul = collection_body("Nul", "1.0.0");
    nul["title"] = json!("bad\u{0}title");
    let (status, answer) = body_of(post_collection(&app, &app.token, &nul).await).await;
    assert_eq!(status, 400);
    assert!(answer["message"].as_str().unwrap().contains("NUL"));

    let mut nul_in_schema = collection_body("Nul schema", "1.0.0");
    nul_in_schema["schemas"] = json!({ "A": { "description": "bad\u{0}" } });
    assert_eq!(
        post_collection(&app, &app.token, &nul_in_schema)
            .await
            .status(),
        400
    );

    let mut tracked = collection_body("Tracked", "1.0.0");
    tracked["source"] = json!({
        "kind": "url",
        "url": "https://api.example.com/openapi.json?token=abc#frag"
    });
    tracked["operations"][0]["id"] = json!("  getAddress  ");
    let stored = {
        let response = post_collection(&app, &app.token, &tracked).await;
        assert_eq!(response.status(), 201);
        let envelope: Value = response.json().await.unwrap();
        envelope["data"].clone()
    };
    assert_eq!(
        stored["source"],
        json!({ "kind": "url", "url": "https://api.example.com/openapi.json" })
    );
    assert_eq!(stored["operations"][0]["id"], "getAddress");

    let mut forbidden_header = collection_body("Header", "1.0.0");
    forbidden_header["operations"][0]["parameters"] = json!([{
        "name": "Host",
        "location": "header",
        "required": false,
        "description": "",
        "schema": { "type": "string" }
    }]);
    let (status, answer) =
        body_of(post_collection(&app, &app.token, &forbidden_header).await).await;
    assert_eq!(status, 400);
    assert!(answer["message"].as_str().unwrap().contains("Host"));

    app.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn a_nested_broken_reference_dot_segment_and_repeated_header_are_refused_when_saving() {
    let app = harness::start().await;
    let mut body = collection_body("Headers", "1.0.0");
    body["operations"][0]["parameters"] = json!([
        { "name": "id", "location": "path", "required": true, "description": "", "schema": {} },
        { "name": "X-Trace", "location": "header", "required": false, "description": "", "schema": {} }
    ]);
    let response = post_collection(&app, &app.token, &body).await;
    assert_eq!(response.status(), 201);
    let collection: Value = response.json().await.unwrap();
    let collection_id = collection["data"]["id"].as_str().unwrap().to_owned();
    let workflow_id = new_workflow(&app).await;

    let nested = save_graph(
        &app,
        &workflow_id,
        operation_graph(
            &collection_id,
            "getAddress",
            json!({ "path": { "id": "{{ connectors.ghost.output.id }}" } }),
        ),
    )
    .await;
    let (status, answer) = body_of(nested).await;
    assert_eq!(status, 422);
    let errors = graph_errors(&answer);
    assert_eq!(errors.len(), 1, "{errors:?}");
    assert_eq!(errors[0].0.as_deref(), Some("c1"));
    assert_eq!(errors[0].1.as_deref(), Some("path.id"));
    assert!(errors[0].2.contains("ghost"), "{}", errors[0].2);

    let dots = save_graph(
        &app,
        &workflow_id,
        operation_graph(
            &collection_id,
            "getAddress",
            json!({ "path": { "id": ".." } }),
        ),
    )
    .await;
    let (status, answer) = body_of(dots).await;
    assert_eq!(status, 422);
    assert_eq!(graph_errors(&answer)[0].1.as_deref(), Some("path.id"));

    let repeated = save_graph(
        &app,
        &workflow_id,
        operation_graph(
            &collection_id,
            "getAddress",
            json!({
                "path": { "id": "1" },
                "headers": { "X-Trace": "a", "x-trace": "b" }
            }),
        ),
    )
    .await;
    let (status, answer) = body_of(repeated).await;
    assert_eq!(status, 422);
    assert_eq!(
        graph_errors(&answer)[0].1.as_deref(),
        Some("headers.x-trace")
    );

    let nested_ok = save_graph(
        &app,
        &workflow_id,
        operation_graph(
            &collection_id,
            "getAddress",
            json!({ "path": { "id": "{{ trigger.id }}" } }),
        ),
    )
    .await;
    assert_eq!(nested_ok.status(), 201);

    app.cleanup().await;
}
