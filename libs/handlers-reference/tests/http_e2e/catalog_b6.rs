use reqwest::StatusCode;
use serde_json::{Value, json};
use uuid::Uuid;

use crate::harness::{self, App};

async fn call(
    app: &App,
    method: reqwest::Method,
    path: &str,
    body: Option<Value>,
) -> (StatusCode, Value) {
    let request = reqwest::Client::new()
        .request(method, app.url(path))
        .bearer_auth(&app.token);
    let request = match body {
        Some(body) => request.json(&body),
        None => request,
    };
    let response = request.send().await.expect("the api answers");
    let status = response.status();
    let text = response.text().await.unwrap_or_default();

    (status, serde_json::from_str(&text).unwrap_or(Value::Null))
}

fn products_path(app: &App) -> String {
    format!("/organizations/{}/products", app.organization_id)
}

fn rates_path(app: &App) -> String {
    format!("/organizations/{}/service-rates", app.organization_id)
}

fn product_body(name: &str) -> Value {
    json!({
        "name": name,
        "unit": "u",
        "unit_price_cents": 1_200,
        "photo_keys": [],
        "category": "Plantes",
        "billing_frequency": "MONTHLY",
        "cost_cents": 700,
        "archived": false,
    })
}

async fn forget(app: &App) {
    for statement in [
        "DELETE FROM quote_lines WHERE org_id = $1",
        "DELETE FROM quotes WHERE org_id = $1",
        "DELETE FROM invoice_lines WHERE org_id = $1",
        "DELETE FROM invoices WHERE org_id = $1",
        "DELETE FROM customer_contexts WHERE customer_id IN (SELECT id FROM customers WHERE org_id = $1)",
        "DELETE FROM customers WHERE org_id = $1",
        "DELETE FROM products WHERE org_id = $1",
        "DELETE FROM service_rates WHERE org_id = $1",
    ] {
        sqlx::query(statement)
            .bind(app.organization_id)
            .execute(&app.pool)
            .await
            .unwrap_or_else(|error| panic!("cleanup failed on `{statement}`: {error}"));
    }
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn spec_crm_6_12_a_product_stores_category_frequency_cost_and_archive_and_refuses_negatives()
{
    let app = harness::start().await;

    let (status, created) = call(
        &app,
        reqwest::Method::POST,
        &products_path(&app),
        Some(product_body("Rosier")),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "{created}");
    let data = &created["data"];
    assert_eq!(data["category"], json!("Plantes"));
    assert_eq!(data["billing_frequency"], json!("MONTHLY"));
    assert_eq!(data["cost_cents"], json!(700));
    assert_eq!(data["archived_at"], Value::Null);
    assert_eq!(data["usage_count"], json!(0));
    let id = data["id"].as_str().expect("an id").to_owned();
    let item_path = format!("/products/{id}");

    let mut archived = product_body("Rosier");
    archived["archived"] = json!(true);
    let (status, updated) = call(&app, reqwest::Method::PATCH, &item_path, Some(archived)).await;
    assert_eq!(status, StatusCode::OK, "{updated}");
    assert!(updated["data"]["archived_at"].is_string(), "{updated}");

    let mut without_catalog = product_body("Rosier");
    for field in ["category", "billing_frequency", "cost_cents", "archived"] {
        without_catalog
            .as_object_mut()
            .expect("an object")
            .remove(field);
    }
    without_catalog["unit_price_cents"] = json!(1_500);
    let (status, kept) = call(
        &app,
        reqwest::Method::PATCH,
        &item_path,
        Some(without_catalog),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{kept}");
    assert_eq!(kept["data"]["unit_price_cents"], json!(1_500));
    assert_eq!(kept["data"]["category"], json!("Plantes"));
    assert_eq!(kept["data"]["cost_cents"], json!(700));
    assert!(kept["data"]["archived_at"].is_string());

    let mut restored = product_body("Rosier");
    restored["cost_cents"] = Value::Null;
    let (status, restored) = call(&app, reqwest::Method::PATCH, &item_path, Some(restored)).await;
    assert_eq!(status, StatusCode::OK, "{restored}");
    assert_eq!(restored["data"]["archived_at"], Value::Null);
    assert_eq!(restored["data"]["cost_cents"], Value::Null);

    let mut negative_price = product_body("Lavande");
    negative_price["unit_price_cents"] = json!(-1);
    let (status, refused) = call(
        &app,
        reqwest::Method::POST,
        &products_path(&app),
        Some(negative_price),
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT, "{refused}");
    let mut negative_cost = product_body("Lavande");
    negative_cost["cost_cents"] = json!(-1);
    let (status, refused) = call(
        &app,
        reqwest::Method::POST,
        &products_path(&app),
        Some(negative_cost),
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT, "{refused}");

    forget(&app).await;
    app.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn spec_crm_6_12_a_service_rate_stores_category_frequency_cost_and_archive() {
    let app = harness::start().await;

    let (status, created) = call(
        &app,
        reqwest::Method::POST,
        &rates_path(&app),
        Some(json!({
            "label": "Entretien annuel",
            "unit": "UNIT",
            "rate_cents": 9_000,
            "category": "Contrats",
            "billing_frequency": "YEARLY",
            "cost_cents": 4_000,
            "archived": true,
        })),
    )
    .await;

    assert_eq!(status, StatusCode::CREATED, "{created}");
    let data = &created["data"];
    assert_eq!(data["category"], json!("Contrats"));
    assert_eq!(data["billing_frequency"], json!("YEARLY"));
    assert_eq!(data["cost_cents"], json!(4_000));
    assert!(data["archived_at"].is_string());

    let (status, refused) = call(
        &app,
        reqwest::Method::POST,
        &rates_path(&app),
        Some(json!({
            "label": "Mauvais",
            "unit": "UNIT",
            "rate_cents": 100,
            "cost_cents": -1,
        })),
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT, "{refused}");

    let (status, listed) = call(
        &app,
        reqwest::Method::GET,
        &format!("{}?page=1&per_page=100", rates_path(&app)),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{listed}");
    assert_eq!(listed["data"][0]["billing_frequency"], json!("YEARLY"));

    forget(&app).await;
    app.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn spec_crm_6_13_editing_an_item_leaves_the_copied_lines_unchanged_and_the_usage_counts_them()
{
    let app = harness::start().await;
    let (_, created) = call(
        &app,
        reqwest::Method::POST,
        &products_path(&app),
        Some(product_body("Rosier")),
    )
    .await;
    let product_id: Uuid = created["data"]["id"]
        .as_str()
        .expect("an id")
        .parse()
        .expect("a uuid");

    let customer_id = Uuid::now_v7();
    sqlx::query("INSERT INTO customers (id, org_id, name) VALUES ($1, $2, 'Lemoine')")
        .bind(customer_id)
        .bind(app.organization_id)
        .execute(&app.pool)
        .await
        .expect("seed the customer");
    let context_id = Uuid::now_v7();
    sqlx::query("INSERT INTO customer_contexts (id, customer_id, label) VALUES ($1, $2, 'Site')")
        .bind(context_id)
        .bind(customer_id)
        .execute(&app.pool)
        .await
        .expect("seed the context");
    let quote_id = Uuid::now_v7();
    sqlx::query(
        "INSERT INTO quotes (id, org_id, customer_id, customer_context_id, status, net_cents, gross_cents, title) \
         VALUES ($1, $2, $3, $4, 'DRAFT'::quote_status, 0, 0, 'Devis')",
    )
    .bind(quote_id)
    .bind(app.organization_id)
    .bind(customer_id)
    .bind(context_id)
    .execute(&app.pool)
    .await
    .expect("seed the quote");
    for _ in 0..2 {
        sqlx::query(
            "INSERT INTO quote_lines (id, org_id, quote_id, product_id, label, quantity, unit, unit_price_cents) \
             VALUES ($1, $2, $3, $4, 'Rosier copie', 1, 'u', 1200)",
        )
        .bind(Uuid::now_v7())
        .bind(app.organization_id)
        .bind(quote_id)
        .bind(product_id)
        .execute(&app.pool)
        .await
        .expect("seed a quote line");
    }
    let invoice_id = Uuid::now_v7();
    sqlx::query(
        "INSERT INTO invoices (id, org_id, kind, customer_id, customer_context_id, status) \
         VALUES ($1, $2, 'STANDARD'::invoice_kind, $3, $4, 'DRAFT'::invoice_status)",
    )
    .bind(invoice_id)
    .bind(app.organization_id)
    .bind(customer_id)
    .bind(context_id)
    .execute(&app.pool)
    .await
    .expect("seed the invoice");
    sqlx::query(
        "INSERT INTO invoice_lines (id, org_id, invoice_id, product_id, label, quantity, unit_price_cents, position) \
         VALUES ($1, $2, $3, $4, 'Rosier copie', 1, 1200, 0)",
    )
    .bind(Uuid::now_v7())
    .bind(app.organization_id)
    .bind(invoice_id)
    .bind(product_id)
    .execute(&app.pool)
    .await
    .expect("seed an invoice line");

    let item_path = format!("/products/{product_id}");
    let (status, read) = call(&app, reqwest::Method::GET, &item_path, None).await;
    assert_eq!(status, StatusCode::OK, "{read}");
    assert_eq!(read["data"]["usage_count"], json!(3));
    assert_eq!(read["data"]["quote_usage_count"], json!(2));
    assert_eq!(read["data"]["invoice_usage_count"], json!(1));

    let mut edited = product_body("Rosier grimpant");
    edited["unit_price_cents"] = json!(9_900);
    let (status, updated) = call(&app, reqwest::Method::PATCH, &item_path, Some(edited)).await;
    assert_eq!(status, StatusCode::OK, "{updated}");
    assert_eq!(updated["data"]["usage_count"], json!(3));

    let quote_line: (String, i32) = sqlx::query_as(
        "SELECT label, unit_price_cents FROM quote_lines WHERE product_id = $1 LIMIT 1",
    )
    .bind(product_id)
    .fetch_one(&app.pool)
    .await
    .expect("the quote line");
    let invoice_line: (String, i32) = sqlx::query_as(
        "SELECT label, unit_price_cents FROM invoice_lines WHERE product_id = $1 LIMIT 1",
    )
    .bind(product_id)
    .fetch_one(&app.pool)
    .await
    .expect("the invoice line");
    assert_eq!(quote_line, ("Rosier copie".to_owned(), 1_200));
    assert_eq!(invoice_line, ("Rosier copie".to_owned(), 1_200));

    forget(&app).await;
    app.cleanup().await;
}
