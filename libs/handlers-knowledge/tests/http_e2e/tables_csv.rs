use reqwest::Method;
use serde_json::{Value, json};
use uuid::Uuid;

use crate::harness::{self, App, Reply};

const UTF8_BOM: &[u8] = b"\xEF\xBB\xBF";
const MAX_CSV_BYTES: usize = 5 * 1024 * 1024;

struct Part<'a> {
    name: &'a str,
    file_name: Option<&'a str>,
    content: &'a [u8],
}

fn file(content: &[u8]) -> Part<'_> {
    Part {
        name: "file",
        file_name: Some("Articles.csv"),
        content,
    }
}

fn multipart(parts: &[Part<'_>]) -> (String, Vec<u8>) {
    let boundary = format!("mestier-{}", Uuid::now_v7().simple());
    let mut body = Vec::new();
    for part in parts {
        body.extend_from_slice(format!("--{boundary}\r\n").as_bytes());
        let disposition = match part.file_name {
            Some(file_name) => format!(
                "Content-Disposition: form-data; name=\"{}\"; filename=\"{file_name}\"\r\nContent-Type: text/csv\r\n\r\n",
                part.name
            ),
            None => format!(
                "Content-Disposition: form-data; name=\"{}\"\r\n\r\n",
                part.name
            ),
        };
        body.extend_from_slice(disposition.as_bytes());
        body.extend_from_slice(part.content);
        body.extend_from_slice(b"\r\n");
    }
    body.extend_from_slice(format!("--{boundary}--\r\n").as_bytes());
    (format!("multipart/form-data; boundary={boundary}"), body)
}

async fn upload(app: &App, token: &str, suffix: &str, parts: &[Part<'_>]) -> Reply {
    let (content_type, body) = multipart(parts);
    let response = reqwest::Client::new()
        .post(app.url(app.organization_id, suffix))
        .bearer_auth(token)
        .header(reqwest::header::CONTENT_TYPE, content_type)
        .body(body)
        .send()
        .await
        .expect("the api answers");
    let status = response.status().as_u16();
    let text = response.text().await.expect("the answer has a body");
    Reply {
        status,
        body: serde_json::from_str(&text).unwrap_or(Value::String(text)),
    }
}

fn windows_1252(text: &str) -> Vec<u8> {
    text.chars()
        .map(|character| match character {
            '€' => 0x80,
            other => u8::try_from(u32::from(other)).expect("a latin-1 character"),
        })
        .collect()
}

fn id(value: &Value) -> String {
    value["id"].as_str().expect("an id").to_owned()
}

async fn create_table(app: &App, name: &str) -> Value {
    let created = app
        .as_manager(Method::POST, "", Some(json!({ "name": name })))
        .await;
    assert_eq!(created.status, 201, "{}", created.body);
    created.body["data"].clone()
}

async fn add_column(app: &App, table: &str, body: Value) -> Value {
    let created = app
        .as_manager(Method::POST, &format!("/{table}/columns"), Some(body))
        .await;
    assert_eq!(created.status, 201, "{}", created.body);
    created.body["data"].clone()
}

async fn add_row(app: &App, table: &str, values: Value) {
    let created = app
        .as_manager(
            Method::POST,
            &format!("/{table}/rows"),
            Some(json!({ "values": values })),
        )
        .await;
    assert_eq!(created.status, 201, "{}", created.body);
}

async fn rows(app: &App, table: &str) -> Value {
    let listed = app
        .as_manager(Method::GET, &format!("/{table}/rows"), None)
        .await;
    assert_eq!(listed.status, 200, "{}", listed.body);
    listed.body["data"].clone()
}

async fn table_count(app: &App) -> usize {
    let listed = app.as_manager(Method::GET, "", None).await;
    assert_eq!(listed.status, 200, "{}", listed.body);
    listed.body["data"].as_array().unwrap().len()
}

async fn row_events(app: &App) -> i64 {
    sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM automation.event WHERE org_id = $1 AND name LIKE 'table_row.%'",
    )
    .bind(app.organization_id)
    .fetch_one(&app.pool)
    .await
    .unwrap()
}

fn many_rows(count: usize) -> Vec<u8> {
    let mut text = String::from("Nom\r\n");
    for n in 0..count {
        text.push_str(&format!("Ligne {n}\r\n"));
    }
    text.into_bytes()
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn a_french_excel_file_becomes_a_table_with_a_number_a_date_and_its_accents() {
    let app = harness::start().await;
    let content = windows_1252(
        "Désignation;Quantité;Livré le;Remarque;Montant\r\n\
         Vis à béton;12,50;05/11/2026;Être livré à l'atelier;1 250,50 €\r\n\
         Écrou;1 250;06/11/2026;;80\r\n",
    );

    let imported = upload(&app, &app.manager.token, "/import", &[file(&content)]).await;

    assert_eq!(imported.status, 201, "{}", imported.body);
    let table = &imported.body["data"]["table"];
    assert_eq!(table["name"], "Articles");
    let columns: Vec<(String, String, bool)> = table["columns"]
        .as_array()
        .unwrap()
        .iter()
        .map(|column| {
            (
                column["name"].as_str().unwrap().to_owned(),
                column["kind"].as_str().unwrap().to_owned(),
                column["is_primary"].as_bool().unwrap(),
            )
        })
        .collect();
    assert_eq!(
        columns,
        vec![
            ("Désignation".to_owned(), "text".to_owned(), true),
            ("Quantité".to_owned(), "number".to_owned(), false),
            ("Livré le".to_owned(), "date".to_owned(), false),
            ("Remarque".to_owned(), "text".to_owned(), false),
            ("Montant".to_owned(), "currency".to_owned(), false),
        ]
    );
    assert_eq!(table["columns"][1]["config"]["decimals"], 2);
    let report = &imported.body["data"]["report"];
    assert_eq!(report["rows_created"], 2);
    assert_eq!(report["invalid_cell_count"], 0);

    let ids: Vec<String> = table["columns"]
        .as_array()
        .unwrap()
        .iter()
        .map(id)
        .collect();
    let listed = rows(&app, &id(table)).await;
    assert_eq!(listed["total"], 2);
    assert_eq!(
        listed["rows"][0]["values"],
        json!({
            ids[0].clone(): "Vis à béton",
            ids[1].clone(): 12.5,
            ids[2].clone(): "2026-11-05",
            ids[3].clone(): "Être livré à l'atelier",
            ids[4].clone(): 125_050,
        })
    );
    assert_eq!(
        listed["rows"][1]["values"],
        json!({ ids[0].clone(): "Écrou", ids[1].clone(): 1250, ids[2].clone(): "2026-11-06", ids[4].clone(): 8_000 })
    );

    let named = upload(
        &app,
        &app.manager.token,
        "/import",
        &[
            file(b"Nom\r\nA\r\n"),
            Part {
                name: "name",
                file_name: None,
                content: "Matériaux".as_bytes(),
            },
        ],
    )
    .await;
    assert_eq!(named.status, 201, "{}", named.body);
    assert_eq!(named.body["data"]["table"]["name"], "Matériaux");
    app.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn an_import_into_a_table_creates_the_rows_and_reports_the_unknown_column_and_the_invalid_date()
 {
    let app = harness::start().await;
    let table = create_table(&app, "Chantiers").await;
    let table_id = id(&table);
    let name = id(&table["columns"][0]);
    let start = id(&add_column(&app, &table_id, json!({ "name": "Début", "kind": "date" })).await);
    let suffix = format!("/{table_id}/import");
    let content = "nom;DÉBUT;Remarque\nVilla;31/02/2026;x\nGrange;2026-11-05;y\n".as_bytes();

    let refused = upload(&app, &app.viewer.token, &suffix, &[file(content)]).await;
    assert_eq!(refused.status, 403, "{}", refused.body);
    assert_eq!(rows(&app, &table_id).await["total"], 0);

    let imported = upload(&app, &app.manager.token, &suffix, &[file(content)]).await;

    assert_eq!(imported.status, 200, "{}", imported.body);
    assert_eq!(
        imported.body["data"],
        json!({
            "rows_created": 2,
            "ignored_columns": ["Remarque"],
            "invalid_cells": [{
                "row": 2,
                "column": "Début",
                "value": "31/02/2026",
                "reason": "not_date",
                "message": "expected a date written AAAA-MM-JJ",
            }],
            "invalid_cell_count": 1,
        })
    );
    let listed = rows(&app, &table_id).await;
    assert_eq!(listed["total"], 2);
    assert_eq!(
        listed["rows"][0]["values"],
        json!({ name.clone(): "Villa" })
    );
    assert_eq!(
        listed["rows"][1]["values"],
        json!({ name: "Grange", start: "2026-11-05" })
    );

    let stranger = upload(
        &app,
        &app.manager.token,
        &format!("/{}/import", Uuid::now_v7()),
        &[file(content)],
    )
    .await;
    assert_eq!(stranger.status, 404, "{}", stranger.body);
    app.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn a_file_over_five_megabytes_or_ten_thousand_rows_is_refused_and_creates_nothing() {
    let app = harness::start().await;
    let mut large = b"Nom\r\n".to_vec();
    large.resize(MAX_CSV_BYTES + 1, b'x');
    let too_many = many_rows(10_001);

    let refused = upload(&app, &app.manager.token, "/import", &[file(&large)]).await;
    assert_eq!(refused.status, 413, "{}", refused.body);
    let refused = upload(&app, &app.manager.token, "/import", &[file(&too_many)]).await;
    assert_eq!(refused.status, 422, "{}", refused.body);
    assert_eq!(table_count(&app).await, 0);

    let table = create_table(&app, "Chantiers").await;
    let table_id = id(&table);
    let suffix = format!("/{table_id}/import");
    for content in [&large, &too_many] {
        let refused = upload(&app, &app.manager.token, &suffix, &[file(content)]).await;
        assert!(
            [413, 422].contains(&refused.status),
            "{}: {}",
            refused.status,
            refused.body
        );
    }
    assert_eq!(rows(&app, &table_id).await["total"], 0);

    sqlx::query(
        r#"INSERT INTO knowledge_table_rows (id, table_id, org_id, "values")
           SELECT gen_random_uuid(), $1, $2, '{}'::jsonb FROM generate_series(1, 9999)"#,
    )
    .bind(table_id.parse::<Uuid>().unwrap())
    .bind(app.organization_id)
    .execute(&app.pool)
    .await
    .unwrap();
    let past_the_cap = upload(&app, &app.manager.token, &suffix, &[file(&many_rows(2))]).await;
    assert_eq!(past_the_cap.status, 422, "{}", past_the_cap.body);
    assert_eq!(rows(&app, &table_id).await["total"], 9_999);
    app.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn an_import_emits_no_row_event() {
    let app = harness::start().await;
    let imported = upload(&app, &app.manager.token, "/import", &[file(&many_rows(50))]).await;
    assert_eq!(imported.status, 201, "{}", imported.body);
    let table_id = id(&imported.body["data"]["table"]);
    let into = upload(
        &app,
        &app.manager.token,
        &format!("/{table_id}/import"),
        &[file(&many_rows(20))],
    )
    .await;
    assert_eq!(into.status, 200, "{}", into.body);
    assert_eq!(rows(&app, &table_id).await["total"], 70);
    assert_eq!(row_events(&app).await, 0);

    let primary = id(&imported.body["data"]["table"]["columns"][0]);
    add_row(&app, &table_id, json!({ primary: "À la main" })).await;
    assert_eq!(row_events(&app).await, 1);
    app.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn the_export_of_a_filtered_view_holds_only_its_rows_and_visible_columns_and_opens_in_excel()
{
    let app = harness::start().await;
    let table = create_table(&app, "Matériaux").await;
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
        json!({ "name": "Statut", "kind": "single_select", "config": { "options": [{ "label": "En cours" }, { "label": "Terminé" }] } }),
    )
    .await;
    let status_id = id(&status);
    let option = |label: &str| {
        status["config"]["options"]
            .as_array()
            .unwrap()
            .iter()
            .find(|option| option["label"] == label)
            .map(id)
            .unwrap()
    };
    let secret =
        id(&add_column(&app, &table_id, json!({ "name": "Secret", "kind": "text" })).await);
    for (label, cents, state) in [
        ("Béton prêt à l'emploi", 125_050, "En cours"),
        ("Sable", 9_900, "Terminé"),
        ("Gravier; lavé", 50_000, "En cours"),
    ] {
        add_row(
            &app,
            &table_id,
            json!({ name.clone(): label, amount.clone(): cents, status_id.clone(): option(state), secret.clone(): "caché" }),
        )
        .await;
    }
    let view = app
        .as_manager(
            Method::POST,
            &format!("/{table_id}/views"),
            Some(json!({
                "name": "En cours",
                "kind": "grid",
                "config": {
                    "filters": [{ "column_id": status_id, "operator": "is", "value": option("En cours") }],
                    "sorts": [{ "column_id": amount, "direction": "asc" }],
                    "hidden": [secret],
                },
            })),
        )
        .await;
    assert_eq!(view.status, 201, "{}", view.body);
    let suffix = format!("/{table_id}/views/{}/export", id(&view.body["data"]));

    let response = reqwest::Client::new()
        .get(app.url(app.organization_id, &suffix))
        .bearer_auth(&app.viewer.token)
        .send()
        .await
        .unwrap();

    assert_eq!(response.status().as_u16(), 200);
    assert_eq!(
        response.headers()["content-type"],
        "text/csv; charset=utf-8"
    );
    let disposition = response.headers()["content-disposition"]
        .to_str()
        .unwrap()
        .to_owned();
    assert!(
        disposition.starts_with(
            "attachment; filename=\"Materiaux - En cours.csv\"; filename*=UTF-8''Mat%C3%A9riaux"
        ),
        "{disposition}"
    );
    let bytes = response.bytes().await.unwrap();
    assert!(bytes.starts_with(UTF8_BOM));
    assert_eq!(
        std::str::from_utf8(&bytes[UTF8_BOM.len()..]).unwrap(),
        "Nom;Montant;Statut\r\n\
         \"Gravier; lavé\";500,00;En cours\r\n\
         Béton prêt à l'emploi;1250,50;En cours\r\n"
    );

    let stranger = app
        .call(
            Method::GET,
            &app.stranger.token,
            app.organization_id,
            &suffix,
            None,
        )
        .await;
    assert_eq!(stranger.status, 403);
    let intruder = app
        .call(
            Method::GET,
            &app.intruder.token,
            app.intruder_organization_id,
            &suffix,
            None,
        )
        .await;
    assert_eq!(intruder.status, 404);
    app.cleanup().await;
}
