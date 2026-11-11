use reqwest::Method;
use serde_json::{Value, json};
use uuid::Uuid;

use crate::harness::{self, App};

async fn create_table(app: &App, name: &str) -> Value {
    let created = app
        .as_manager(Method::POST, "", Some(json!({ "name": name })))
        .await;
    assert_eq!(created.status, 201, "{}", created.body);
    created.body["data"].clone()
}

fn id(value: &Value) -> String {
    value["id"].as_str().expect("an id").to_owned()
}

async fn add_column(app: &App, table: &str, body: Value) -> Value {
    let created = app
        .as_manager(Method::POST, &format!("/{table}/columns"), Some(body))
        .await;
    assert_eq!(created.status, 201, "{}", created.body);
    created.body["data"].clone()
}

fn option(column: &Value, label: &str) -> String {
    column["config"]["options"]
        .as_array()
        .expect("a choice column lists its options")
        .iter()
        .find(|option| option["label"] == label)
        .map(id)
        .expect("the option exists")
}

fn choices(labels: &[&str]) -> Value {
    json!({ "options": labels.iter().map(|label| json!({ "label": label })).collect::<Vec<_>>() })
}

async fn add_row(app: &App, table: &str, values: Value) -> Value {
    let created = app
        .as_manager(
            Method::POST,
            &format!("/{table}/rows"),
            Some(json!({ "values": values })),
        )
        .await;
    assert_eq!(created.status, 201, "{}", created.body);
    created.body["data"].clone()
}

async fn row(app: &App, table: &str, row: &str) -> Value {
    let read = app
        .as_manager(Method::GET, &format!("/{table}/rows/{row}"), None)
        .await;
    assert_eq!(read.status, 200, "{}", read.body);
    read.body["data"].clone()
}

async fn total(app: &App, table: &str) -> u64 {
    let listed = app
        .as_manager(Method::GET, &format!("/{table}/rows"), None)
        .await;
    assert_eq!(listed.status, 200, "{}", listed.body);
    listed.body["data"]["total"].as_u64().unwrap()
}

fn titles(page: &Value, name: &str) -> Vec<String> {
    page["data"]["rows"]
        .as_array()
        .unwrap()
        .iter()
        .map(|row| row["values"][name].as_str().unwrap().to_owned())
        .collect()
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn a_new_table_has_a_primary_text_column_and_a_grid_view() {
    let app = harness::start().await;
    let table = create_table(&app, "Chantiers").await;
    assert_eq!(table["name"], "Chantiers");
    assert_eq!(table["columns"].as_array().unwrap().len(), 1);
    assert_eq!(table["columns"][0]["name"], "Nom");
    assert_eq!(table["columns"][0]["kind"], "text");
    assert_eq!(table["columns"][0]["is_primary"], true);
    assert_eq!(table["views"].as_array().unwrap().len(), 1);
    assert_eq!(table["views"][0]["kind"], "grid");

    let listed = app.as_manager(Method::GET, "", None).await;
    assert_eq!(listed.status, 200);
    assert_eq!(listed.body["data"][0]["id"], table["id"]);
    app.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn a_row_with_a_value_of_every_kind_reads_back_the_same_and_survives_a_deleted_customer() {
    let app = harness::start().await;
    let table = create_table(&app, "Chantiers").await;
    let table_id = id(&table);
    let name = id(&table["columns"][0]);
    let column = |name: &str, kind: &str, config: Value| json!({ "name": name, "kind": kind, "config": config });
    let mut ids = vec![name.clone()];
    let mut columns = Vec::new();
    for body in [
        column("Description", "long_text", json!({})),
        column("Quantité", "number", json!({ "decimals": 2 })),
        column("Montant", "currency", json!({})),
        column("Début", "date", json!({})),
        column("Fait", "checkbox", json!({})),
        column("Statut", "single_select", choices(&["En cours", "Terminé"])),
        column("Tags", "multi_select", choices(&["Urgent", "Client"])),
        column("Client", "link", json!({ "link_kind": "customer" })),
        column("Fichier", "file", json!({})),
        column("Responsable", "member", json!({})),
    ] {
        let created = add_column(&app, &table_id, body).await;
        ids.push(id(&created));
        columns.push(created);
    }
    let customer = app.seed_customer(app.organization_id, "Dupont SARL").await;
    let file = app.seed_file(app.organization_id, "Plan de masse").await;
    let values = json!({
        ids[0].clone(): "Villa Dupont",
        ids[1].clone(): "Terrasse en bois\nsur plots",
        ids[2].clone(): 12.5,
        ids[3].clone(): 150_000,
        ids[4].clone(): "2026-11-05",
        ids[5].clone(): true,
        ids[6].clone(): option(&columns[5], "En cours"),
        ids[7].clone(): [option(&columns[6], "Urgent"), option(&columns[6], "Client")],
        ids[8].clone(): { "kind": "customer", "id": customer },
        ids[9].clone(): file,
        ids[10].clone(): app.manager.user_id,
    });
    let created = add_row(&app, &table_id, values.clone()).await;
    let read = row(&app, &table_id, &id(&created)).await;
    assert_eq!(read["values"], values);
    assert_eq!(read["labels"][&ids[8]]["label"], "Dupont SARL");
    assert_eq!(read["labels"][&ids[9]]["label"], "Plan de masse");
    assert_eq!(read["labels"][&ids[10]]["label"], "Léa Martin");
    assert_eq!(read["labels"][&ids[8]]["missing"], false);

    sqlx::query("UPDATE customers SET deleted_at = now() WHERE id = $1")
        .bind(customer)
        .execute(&app.pool)
        .await
        .unwrap();
    let after = row(&app, &table_id, &id(&created)).await;
    assert_eq!(after["values"], values);
    assert_eq!(after["labels"][&ids[8]]["missing"], true);
    assert_eq!(after["labels"][&ids[8]]["label"], Value::Null);
    app.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn an_invalid_value_is_refused_and_nothing_is_stored() {
    let app = harness::start().await;
    let table = create_table(&app, "Chantiers").await;
    let table_id = id(&table);
    let quantity = id(&add_column(
        &app,
        &table_id,
        json!({ "name": "Quantité", "kind": "number" }),
    )
    .await);
    let status = id(&add_column(
        &app,
        &table_id,
        json!({ "name": "Statut", "kind": "single_select", "config": choices(&["En cours"]) }),
    )
    .await);
    let start = id(&add_column(&app, &table_id, json!({ "name": "Début", "kind": "date" })).await);
    let client = id(&add_column(
        &app,
        &table_id,
        json!({ "name": "Client", "kind": "link", "config": { "link_kind": "customer" } }),
    )
    .await);
    let foreign = app
        .seed_customer(app.intruder_organization_id, "Ailleurs SA")
        .await;
    for values in [
        json!({ quantity.clone(): "douze" }),
        json!({ status.clone(): Uuid::now_v7() }),
        json!({ start.clone(): "31/02/2026" }),
        json!({ start.clone(): "2026-02-31" }),
        json!({ client.clone(): { "kind": "customer", "id": foreign } }),
        json!({ Uuid::now_v7().to_string(): "x" }),
    ] {
        let refused = app
            .as_manager(
                Method::POST,
                &format!("/{table_id}/rows"),
                Some(json!({ "values": values })),
            )
            .await;
        assert_eq!(refused.status, 422, "{values}: {}", refused.body);
    }
    assert_eq!(total(&app, &table_id).await, 0);
    app.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn two_edits_on_two_cells_from_the_same_state_keep_both() {
    let app = harness::start().await;
    let table = create_table(&app, "Chantiers").await;
    let table_id = id(&table);
    let name = id(&table["columns"][0]);
    let done = id(&add_column(
        &app,
        &table_id,
        json!({ "name": "Fait", "kind": "checkbox" }),
    )
    .await);
    let created = add_row(&app, &table_id, json!({ name.clone(): "Villa" })).await;
    let path = format!("/{table_id}/rows/{}", id(&created));
    for values in [
        json!({ done.clone(): true }),
        json!({ name.clone(): "Villa Dupont" }),
    ] {
        let patched = app
            .as_manager(Method::PATCH, &path, Some(json!({ "values": values })))
            .await;
        assert_eq!(patched.status, 200, "{}", patched.body);
    }
    let read = row(&app, &table_id, &id(&created)).await;
    assert_eq!(read["values"], json!({ name: "Villa Dupont", done: true }));
    app.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn deleting_a_column_or_an_option_removes_it_from_the_rows() {
    let app = harness::start().await;
    let table = create_table(&app, "Chantiers").await;
    let table_id = id(&table);
    let name = id(&table["columns"][0]);
    let status = add_column(
        &app,
        &table_id,
        json!({ "name": "Statut", "kind": "single_select", "config": choices(&["En cours", "Terminé"]) }),
    )
    .await;
    let tags = id(&add_column(
        &app,
        &table_id,
        json!({ "name": "Tags", "kind": "multi_select", "config": choices(&["Urgent"]) }),
    )
    .await);
    let status_id = id(&status);
    let running = option(&status, "En cours");
    let done = option(&status, "Terminé");
    let tags_column = app
        .as_manager(Method::GET, &format!("/{table_id}"), None)
        .await
        .body["data"]["columns"][2]
        .clone();
    let urgent = option(&tags_column, "Urgent");
    let first = add_row(
        &app,
        &table_id,
        json!({ name.clone(): "A", status_id.clone(): running, tags.clone(): [urgent] }),
    )
    .await;
    let second = add_row(
        &app,
        &table_id,
        json!({ name.clone(): "B", status_id.clone(): done }),
    )
    .await;

    let deleted = app
        .as_manager(Method::DELETE, &format!("/{table_id}/columns/{tags}"), None)
        .await;
    assert_eq!(deleted.status, 204);
    let kept = app
        .as_manager(
            Method::PATCH,
            &format!("/{table_id}/columns/{status_id}"),
            Some(json!({ "config": { "options": [{ "id": done, "label": "Terminé" }] } })),
        )
        .await;
    assert_eq!(kept.status, 200, "{}", kept.body);

    let first = row(&app, &table_id, &id(&first)).await;
    assert_eq!(first["values"], json!({ name.clone(): "A" }));
    let second = row(&app, &table_id, &id(&second)).await;
    assert_eq!(second["values"], json!({ name: "B", status_id: done }));
    app.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn the_primary_column_and_the_last_view_cannot_be_deleted() {
    let app = harness::start().await;
    let table = create_table(&app, "Chantiers").await;
    let table_id = id(&table);
    let primary = id(&table["columns"][0]);
    let grid = id(&table["views"][0]);
    let refused = app
        .as_manager(
            Method::DELETE,
            &format!("/{table_id}/columns/{primary}"),
            None,
        )
        .await;
    assert_eq!(refused.status, 409, "{}", refused.body);
    let refused = app
        .as_manager(Method::DELETE, &format!("/{table_id}/views/{grid}"), None)
        .await;
    assert_eq!(refused.status, 409, "{}", refused.body);

    let other = app
        .as_manager(
            Method::POST,
            &format!("/{table_id}/views"),
            Some(json!({ "name": "Autre", "kind": "grid" })),
        )
        .await;
    assert_eq!(other.status, 201, "{}", other.body);
    let deleted = app
        .as_manager(Method::DELETE, &format!("/{table_id}/views/{grid}"), None)
        .await;
    assert_eq!(deleted.status, 204);
    app.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn only_lossless_type_changes_are_applied() {
    let app = harness::start().await;
    let table = create_table(&app, "Chantiers").await;
    let table_id = id(&table);
    let quantity = id(&add_column(
        &app,
        &table_id,
        json!({ "name": "Quantité", "kind": "number" }),
    )
    .await);
    let tags = add_column(
        &app,
        &table_id,
        json!({ "name": "Tags", "kind": "multi_select", "config": choices(&["A", "B"]) }),
    )
    .await;
    let tags_id = id(&tags);
    let both = json!([option(&tags, "A"), option(&tags, "B")]);
    let created = add_row(
        &app,
        &table_id,
        json!({ quantity.clone(): 1250.5, tags_id.clone(): both.clone() }),
    )
    .await;

    let to_text = app
        .as_manager(
            Method::PATCH,
            &format!("/{table_id}/columns/{quantity}"),
            Some(json!({ "kind": "text" })),
        )
        .await;
    assert_eq!(to_text.status, 200, "{}", to_text.body);
    assert_eq!(to_text.body["data"]["kind"], "text");

    let to_single = app
        .as_manager(
            Method::PATCH,
            &format!("/{table_id}/columns/{tags_id}"),
            Some(json!({ "kind": "single_select" })),
        )
        .await;
    assert_eq!(to_single.status, 409, "{}", to_single.body);

    let read = row(&app, &table_id, &id(&created)).await;
    assert_eq!(read["values"][&quantity], "1250,5");
    assert_eq!(read["values"][&tags_id], both);
    let schema = app
        .as_manager(Method::GET, &format!("/{table_id}"), None)
        .await;
    assert_eq!(schema.body["data"]["columns"][2]["kind"], "multi_select");

    let price = id(&add_column(&app, &table_id, json!({ "name": "Prix", "kind": "number" })).await);
    let to_date = app
        .as_manager(
            Method::PATCH,
            &format!("/{table_id}/columns/{price}"),
            Some(json!({ "kind": "date" })),
        )
        .await;
    assert_eq!(to_date.status, 422, "{}", to_date.body);
    app.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn rows_are_filtered_sorted_and_paged_ad_hoc_and_through_a_view() {
    let app = harness::start().await;
    let table = create_table(&app, "Chantiers").await;
    let table_id = id(&table);
    let name = id(&table["columns"][0]);
    let amount = id(&add_column(
        &app,
        &table_id,
        json!({ "name": "Montant", "kind": "currency" }),
    )
    .await);
    let status = add_column(
        &app,
        &table_id,
        json!({ "name": "Statut", "kind": "single_select", "config": choices(&["En cours", "Terminé"]) }),
    )
    .await;
    let status_id = id(&status);
    let start = id(&add_column(&app, &table_id, json!({ "name": "Début", "kind": "date" })).await);
    let running = option(&status, "En cours");
    let done = option(&status, "Terminé");
    for (label, cents, state, date) in [
        ("A", 250_000, &running, "2026-03-01"),
        ("B", 50_000, &running, "2026-04-01"),
        ("C", 300_000, &done, "2026-05-01"),
        ("D", 150_000, &running, "2026-06-01"),
        ("E", 120_000, &running, "2026-01-15"),
    ] {
        add_row(
            &app,
            &table_id,
            json!({ name.clone(): label, amount.clone(): cents, status_id.clone(): state, start.clone(): date }),
        )
        .await;
    }
    let filters = json!([
        { "column_id": amount, "operator": "gt", "value": 100_000 },
        { "column_id": status_id, "operator": "is", "value": running },
    ]);
    let sorts = json!([{ "column_id": start, "direction": "desc" }]);
    let query =
        |offset: u64| json!({ "filters": filters, "sorts": sorts, "limit": 2, "offset": offset });
    let first = app
        .as_manager(
            Method::POST,
            &format!("/{table_id}/rows/query"),
            Some(query(0)),
        )
        .await;
    assert_eq!(first.status, 200, "{}", first.body);
    assert_eq!(first.body["data"]["total"], 3);
    assert_eq!(titles(&first.body, &name), vec!["D", "A"]);
    let second = app
        .as_manager(
            Method::POST,
            &format!("/{table_id}/rows/query"),
            Some(query(2)),
        )
        .await;
    assert_eq!(titles(&second.body, &name), vec!["E"]);

    let view = app
        .as_manager(
            Method::POST,
            &format!("/{table_id}/views"),
            Some(json!({
                "name": "Gros chantiers en cours",
                "kind": "kanban",
                "config": { "filters": filters, "sorts": sorts, "group_by": status_id },
            })),
        )
        .await;
    assert_eq!(view.status, 201, "{}", view.body);
    let through_view = app
        .as_manager(
            Method::GET,
            &format!(
                "/{table_id}/rows?view_id={}&limit=2&offset=0",
                id(&view.body["data"])
            ),
            None,
        )
        .await;
    assert_eq!(through_view.status, 200, "{}", through_view.body);
    assert_eq!(titles(&through_view.body, &name), vec!["D", "A"]);

    let wrong = app
        .as_manager(
            Method::POST,
            &format!("/{table_id}/rows/query"),
            Some(json!({ "filters": [{ "column_id": amount, "operator": "contains", "value": "1" }] })),
        )
        .await;
    assert_eq!(wrong.status, 422, "{}", wrong.body);

    let client = id(&add_column(
        &app,
        &table_id,
        json!({ "name": "Client", "kind": "link", "config": { "link_kind": "customer" } }),
    )
    .await);
    let unsortable = app
        .as_manager(
            Method::POST,
            &format!("/{table_id}/rows/query"),
            Some(json!({ "sorts": [{ "column_id": client, "direction": "asc" }] })),
        )
        .await;
    assert_eq!(unsortable.status, 422, "{}", unsortable.body);
    app.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn the_bounds_and_unique_names_are_enforced() {
    let app = harness::start().await;
    let table = create_table(&app, "Chantiers").await;
    let table_id = id(&table);
    add_column(&app, &table_id, json!({ "name": "Statut", "kind": "text" })).await;
    let twin = app
        .as_manager(
            Method::POST,
            &format!("/{table_id}/columns"),
            Some(json!({ "name": "statut", "kind": "checkbox" })),
        )
        .await;
    assert_eq!(twin.status, 409, "{}", twin.body);
    for n in 0..48 {
        add_column(
            &app,
            &table_id,
            json!({ "name": format!("C{n}"), "kind": "text" }),
        )
        .await;
    }
    let fifty_first = app
        .as_manager(
            Method::POST,
            &format!("/{table_id}/columns"),
            Some(json!({ "name": "Une de trop", "kind": "text" })),
        )
        .await;
    assert_eq!(fifty_first.status, 422, "{}", fifty_first.body);

    let oversized = app
        .as_manager(
            Method::POST,
            &format!("/{table_id}/rows"),
            Some(json!({ "values": {}, "padding": "x".repeat(300 * 1024) })),
        )
        .await;
    assert_eq!(oversized.status, 413);

    let twin_table = app
        .as_manager(Method::POST, "", Some(json!({ "name": "CHANTIERS" })))
        .await;
    assert_eq!(twin_table.status, 409, "{}", twin_table.body);

    sqlx::query(
        r#"INSERT INTO knowledge_table_rows (id, table_id, org_id, "values")
           SELECT gen_random_uuid(), $1, $2, '{}'::jsonb FROM generate_series(1, 10000)"#,
    )
    .bind(table_id.parse::<Uuid>().unwrap())
    .bind(app.organization_id)
    .execute(&app.pool)
    .await
    .unwrap();
    let refused = app
        .as_manager(
            Method::POST,
            &format!("/{table_id}/rows"),
            Some(json!({ "values": {} })),
        )
        .await;
    assert_eq!(refused.status, 422, "{}", refused.body);
    assert_eq!(total(&app, &table_id).await, 10_000);
    app.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn another_organization_finds_nothing_and_changes_nothing() {
    let app = harness::start().await;
    let table = create_table(&app, "Chantiers").await;
    let table_id = id(&table);
    let name = id(&table["columns"][0]);
    let grid = id(&table["views"][0]);
    let created = add_row(&app, &table_id, json!({ name.clone(): "Villa" })).await;
    let row_id = id(&created);
    let intruder = |method: Method, suffix: String, payload: Option<Value>| {
        let app = &app;
        async move {
            app.call(
                method,
                &app.intruder.token,
                app.intruder_organization_id,
                &suffix,
                payload,
            )
            .await
        }
    };
    let attempts = [
        (Method::GET, format!("/{table_id}"), None),
        (
            Method::PATCH,
            format!("/{table_id}"),
            Some(json!({ "name": "Volé" })),
        ),
        (Method::DELETE, format!("/{table_id}"), None),
        (
            Method::POST,
            format!("/{table_id}/columns"),
            Some(json!({ "name": "X", "kind": "text" })),
        ),
        (
            Method::PATCH,
            format!("/{table_id}/columns/{name}"),
            Some(json!({ "name": "Volé" })),
        ),
        (Method::DELETE, format!("/{table_id}/columns/{name}"), None),
        (
            Method::PUT,
            format!("/{table_id}/columns/order"),
            Some(json!({ "order": [name] })),
        ),
        (
            Method::POST,
            format!("/{table_id}/views"),
            Some(json!({ "name": "X", "kind": "grid" })),
        ),
        (
            Method::PATCH,
            format!("/{table_id}/views/{grid}"),
            Some(json!({ "name": "Volé" })),
        ),
        (Method::DELETE, format!("/{table_id}/views/{grid}"), None),
        (Method::GET, format!("/{table_id}/rows"), None),
        (
            Method::POST,
            format!("/{table_id}/rows/query"),
            Some(json!({})),
        ),
        (
            Method::POST,
            format!("/{table_id}/rows"),
            Some(json!({ "values": {} })),
        ),
        (Method::GET, format!("/{table_id}/rows/{row_id}"), None),
        (
            Method::PATCH,
            format!("/{table_id}/rows/{row_id}"),
            Some(json!({ "values": { name.clone(): "Volé" } })),
        ),
        (Method::DELETE, format!("/{table_id}/rows/{row_id}"), None),
        (
            Method::POST,
            format!("/{table_id}/rows/delete"),
            Some(json!({ "ids": [row_id] })),
        ),
    ];
    for (method, suffix, payload) in attempts {
        let reply = intruder(method.clone(), suffix.clone(), payload).await;
        assert_eq!(reply.status, 404, "{method} {suffix}: {}", reply.body);
    }
    let listed = intruder(Method::GET, String::new(), None).await;
    assert_eq!(listed.body["data"], json!([]));
    let through_foreign_url = app
        .call(
            Method::GET,
            &app.intruder.token,
            app.organization_id,
            &format!("/{table_id}"),
            None,
        )
        .await;
    assert_eq!(through_foreign_url.status, 403);

    let schema = app
        .as_manager(Method::GET, &format!("/{table_id}"), None)
        .await;
    assert_eq!(schema.body["data"]["name"], "Chantiers");
    assert_eq!(schema.body["data"]["columns"][0]["name"], "Nom");
    assert_eq!(schema.body["data"]["views"][0]["name"], "Grille");
    assert_eq!(
        row(&app, &table_id, &row_id).await["values"],
        json!({ name: "Villa" })
    );
    app.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn writing_needs_manage_knowledge_and_reading_view_knowledge() {
    let app = harness::start().await;
    let table = create_table(&app, "Chantiers").await;
    let table_id = id(&table);
    let name = id(&table["columns"][0]);
    let grid = id(&table["views"][0]);
    let row_id = id(&add_row(&app, &table_id, json!({ name.clone(): "Villa" })).await);
    let as_caller = |token: &str, method: Method, suffix: String, payload: Option<Value>| {
        let app = &app;
        let token = token.to_owned();
        async move {
            app.call(method, &token, app.organization_id, &suffix, payload)
                .await
        }
    };

    let reads = [
        (Method::GET, String::new(), None),
        (Method::GET, format!("/{table_id}"), None),
        (Method::GET, format!("/{table_id}/rows"), None),
        (
            Method::POST,
            format!("/{table_id}/rows/query"),
            Some(json!({})),
        ),
        (Method::GET, format!("/{table_id}/rows/{row_id}"), None),
    ];
    for (method, suffix, payload) in reads {
        let viewer = as_caller(
            &app.viewer.token,
            method.clone(),
            suffix.clone(),
            payload.clone(),
        )
        .await;
        assert_eq!(
            viewer.status, 200,
            "viewer {method} {suffix}: {}",
            viewer.body
        );
        let stranger =
            as_caller(&app.stranger.token, method.clone(), suffix.clone(), payload).await;
        assert_eq!(stranger.status, 403, "stranger {method} {suffix}");
    }

    let writes = [
        (
            Method::POST,
            String::new(),
            Some(json!({ "name": "Autre" })),
        ),
        (
            Method::PATCH,
            format!("/{table_id}"),
            Some(json!({ "name": "Renommée" })),
        ),
        (Method::DELETE, format!("/{table_id}"), None),
        (
            Method::POST,
            format!("/{table_id}/columns"),
            Some(json!({ "name": "X", "kind": "text" })),
        ),
        (
            Method::PATCH,
            format!("/{table_id}/columns/{name}"),
            Some(json!({ "name": "Titre" })),
        ),
        (Method::DELETE, format!("/{table_id}/columns/{name}"), None),
        (
            Method::PUT,
            format!("/{table_id}/columns/order"),
            Some(json!({ "order": [name] })),
        ),
        (
            Method::POST,
            format!("/{table_id}/views"),
            Some(json!({ "name": "X", "kind": "grid" })),
        ),
        (
            Method::PATCH,
            format!("/{table_id}/views/{grid}"),
            Some(json!({ "name": "Y" })),
        ),
        (Method::DELETE, format!("/{table_id}/views/{grid}"), None),
        (
            Method::POST,
            format!("/{table_id}/rows"),
            Some(json!({ "values": {} })),
        ),
        (
            Method::PATCH,
            format!("/{table_id}/rows/{row_id}"),
            Some(json!({ "values": { name.clone(): "Non" } })),
        ),
        (Method::DELETE, format!("/{table_id}/rows/{row_id}"), None),
        (
            Method::POST,
            format!("/{table_id}/rows/delete"),
            Some(json!({ "ids": [row_id] })),
        ),
    ];
    for (method, suffix, payload) in writes {
        for token in [&app.viewer.token, &app.stranger.token] {
            let reply = as_caller(token, method.clone(), suffix.clone(), payload.clone()).await;
            assert_eq!(reply.status, 403, "{method} {suffix}: {}", reply.body);
        }
    }
    assert_eq!(
        row(&app, &table_id, &row_id).await["values"],
        json!({ name: "Villa" })
    );
    app.cleanup().await;
}
