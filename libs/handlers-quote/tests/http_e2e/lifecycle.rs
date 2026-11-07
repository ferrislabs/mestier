use chrono::{Duration, Utc};
use serde_json::{Value, json};
use uuid::Uuid;

use crate::harness::App;

fn line(label: &str, price: i32) -> Value {
    json!({ "label": label, "quantity": "1", "unit": "HOUR", "unit_price_cents": price, "photo_keys": [] })
}

async fn post_quote(app: &App, body: Value) -> reqwest::Response {
    reqwest::Client::new()
        .post(app.quotes_url())
        .bearer_auth(&app.token)
        .json(&body)
        .send()
        .await
        .expect("the api answers the create call")
}

fn quote_body(app: &App, lines: Vec<Value>) -> Value {
    json!({
        "title": "Kitchen renovation",
        "customer_id": app.customer_id,
        "customer_context_id": app.customer_context_id,
        "lines": lines,
    })
}

async fn create(app: &App, body: Value) -> Value {
    let response = post_quote(app, body).await;
    let status = response.status();
    let envelope: Value = response.json().await.expect("a json answer");
    assert_eq!(status, 201, "{envelope}");
    envelope["data"].clone()
}

async fn set_status(app: &App, id: &str, body: Value) -> (u16, Value) {
    let response = reqwest::Client::new()
        .patch(format!("{}/api/v1/quotes/{id}/status", app.base_url))
        .bearer_auth(&app.token)
        .json(&body)
        .send()
        .await
        .expect("the api answers the status call");
    let status = response.status().as_u16();

    (status, response.json().await.expect("a json answer"))
}

async fn get(app: &App, id: &str) -> Value {
    let response = reqwest::Client::new()
        .get(format!("{}/api/v1/quotes/{id}", app.base_url))
        .bearer_auth(&app.token)
        .send()
        .await
        .expect("the api answers the get call");
    assert_eq!(response.status(), 200);
    let envelope: Value = response.json().await.expect("a json answer");

    envelope["data"].clone()
}

async fn send(app: &App, id: &str) -> Value {
    let (status, body) = set_status(app, id, json!({ "status": "SENT" })).await;
    assert_eq!(status, 200, "{body}");

    body["data"].clone()
}

fn id_of(quote: &Value) -> String {
    quote["id"].as_str().expect("an id").to_owned()
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn spec_crm_4_3_to_4_6_a_quote_is_sent_frozen_and_accepted_over_http_and_wins_its_deal() {
    let app = crate::harness::start().await;
    let deal_id = Uuid::now_v7();
    sqlx::query(
        "INSERT INTO deals (id, org_id, customer_id, title, amount_cents, stage) VALUES ($1, $2, $3, 'Cuisine Duval', 0, 'NEW')",
    )
    .bind(deal_id)
    .bind(app.organization_id)
    .bind(app.customer_id)
    .execute(&app.pool)
    .await
    .expect("seed the deal");

    let mut body = quote_body(
        &app,
        vec![
            json!({ "label": "Pose", "quantity": "3", "unit": "HOUR", "unit_price_cents": 10000, "discount_percent": "10", "photo_keys": [] }),
            json!({ "label": "Entretien", "quantity": "1", "unit": "FLAT_RATE", "unit_price_cents": 2000, "billing_frequency": "MONTHLY", "photo_keys": [] }),
        ],
    );
    body["deal_id"] = json!(deal_id);
    body["deposit_percent"] = json!("30");
    body["notes"] = json!("Accès par la cour");
    body["owner_member_id"] = json!(app.member_id);
    let created = create(&app, body).await;
    let quote_id = id_of(&created);

    assert_eq!(created["net_cents"], json!(29_000));
    assert_eq!(created["lines"][0]["discount_percent"], json!("10"));
    assert_eq!(created["lines"][0]["net_cents"], json!(27_000));
    assert_eq!(created["deposit_percent"], json!("30"));
    assert_eq!(created["owner_member_id"], json!(app.member_id));
    assert_eq!(created["notes"], json!("Accès par la cour"));
    assert_eq!(
        created["recurring"],
        json!([{ "frequency": "MONTHLY", "net_cents": 2000, "vat_cents": 0, "gross_cents": 2000 }])
    );
    assert_eq!(created["is_expired"], json!(false));
    assert_eq!(created["issuer_snapshot"], json!(null));

    let sent = send(&app, &quote_id).await;

    let today = Utc::now().date_naive();
    assert_eq!(sent["status"], json!("SENT"));
    assert!(sent["reference"].as_str().unwrap().starts_with("DEV-"));
    assert_eq!(
        sent["valid_until"],
        json!((today + Duration::days(30)).to_string())
    );
    assert_eq!(sent["issuer_snapshot"]["name"], json!("Test Org"));
    assert_eq!(sent["recipient_snapshot"]["name"], json!("Duval Masonry"));
    assert_eq!(sent["recipient_snapshot"]["address"]["city"], json!("Lyon"));

    let (stage, amount): (String, i64) =
        sqlx::query_as("SELECT stage, amount_cents FROM deals WHERE id = $1")
            .bind(deal_id)
            .fetch_one(&app.pool)
            .await
            .expect("read the deal");
    assert_eq!((stage.as_str(), amount), ("QUOTE_SENT", 29_000));

    let (status, accepted) = set_status(
        &app,
        &quote_id,
        json!({ "status": "ACCEPTED", "responded_by": "  Mme Duval  " }),
    )
    .await;
    assert_eq!(status, 200, "{accepted}");
    assert_eq!(accepted["data"]["status"], json!("ACCEPTED"));
    assert_eq!(accepted["data"]["responded_by"], json!("Mme Duval"));
    assert!(accepted["data"]["responded_at"].is_string());

    let stage: String = sqlx::query_scalar("SELECT stage FROM deals WHERE id = $1")
        .bind(deal_id)
        .fetch_one(&app.pool)
        .await
        .expect("read the deal");
    let customer_status: String = sqlx::query_scalar("SELECT status FROM customers WHERE id = $1")
        .bind(app.customer_id)
        .fetch_one(&app.pool)
        .await
        .expect("read the customer");
    assert_eq!(stage, "WON");
    assert_eq!(customer_status, "CLIENT");

    app.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn spec_crm_4_1_a_sent_quote_is_locked_and_only_a_draft_is_deleted() {
    let app = crate::harness::start().await;
    let created = create(&app, quote_body(&app, vec![line("Pose", 4500)])).await;
    let quote_id = id_of(&created);
    send(&app, &quote_id).await;

    let edit = reqwest::Client::new()
        .patch(format!("{}/api/v1/quotes/{quote_id}", app.base_url))
        .bearer_auth(&app.token)
        .json(&json!({
            "title": "Autre",
            "customer_id": app.customer_id,
            "customer_context_id": app.customer_context_id,
            "status": "SENT",
            "lines": [line("Pose", 4500)],
        }))
        .send()
        .await
        .expect("the api answers the update call");
    assert_eq!(edit.status(), 409);
    let body: Value = edit.json().await.unwrap();
    assert_eq!(body["code"], json!("E_QUOTE_LOCKED"));
    assert_eq!(body["details"]["field"], json!("status"));

    let delete = reqwest::Client::new()
        .delete(format!("{}/api/v1/quotes/{quote_id}", app.base_url))
        .bearer_auth(&app.token)
        .send()
        .await
        .expect("the api answers the delete call");
    assert_eq!(delete.status(), 409);
    assert_eq!(
        delete.json::<Value>().await.unwrap()["code"],
        json!("E_QUOTE_LOCKED")
    );

    let draft = create(&app, quote_body(&app, vec![line("Pose", 4500)])).await;
    let removed = reqwest::Client::new()
        .delete(format!("{}/api/v1/quotes/{}", app.base_url, id_of(&draft)))
        .bearer_auth(&app.token)
        .send()
        .await
        .expect("the api answers the delete call");
    assert_eq!(removed.status(), 204);

    app.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn spec_crm_4_5_to_4_7_answering_needs_a_sent_quote_and_accepting_an_expired_one_is_refused()
{
    let app = crate::harness::start().await;
    let draft = create(&app, quote_body(&app, vec![line("Pose", 4500)])).await;

    let (status, body) = set_status(&app, &id_of(&draft), json!({ "status": "ACCEPTED" })).await;
    assert_eq!(status, 409);
    assert_eq!(body["code"], json!("E_QUOTE_NOT_SENT"));

    let sent = send(&app, &id_of(&draft)).await;
    let quote_id = id_of(&sent);
    sqlx::query("UPDATE quotes SET valid_until = CURRENT_DATE - 1 WHERE id = $1::uuid")
        .bind(&quote_id)
        .execute(&app.pool)
        .await
        .expect("backdate the validity");
    assert_eq!(get(&app, &quote_id).await["is_expired"], json!(true));

    let (status, body) = set_status(&app, &quote_id, json!({ "status": "ACCEPTED" })).await;
    assert_eq!(status, 409);
    assert_eq!(body["code"], json!("E_QUOTE_EXPIRED"));
    assert_eq!(get(&app, &quote_id).await["status"], json!("SENT"));

    let (status, declined) = set_status(
        &app,
        &quote_id,
        json!({ "status": "DECLINED", "decline_reason": " Trop cher " }),
    )
    .await;
    assert_eq!(status, 200, "{declined}");
    assert_eq!(declined["data"]["status"], json!("DECLINED"));
    assert_eq!(declined["data"]["decline_reason"], json!("Trop cher"));
    assert_eq!(declined["data"]["is_expired"], json!(false));

    app.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn spec_crm_4_8_and_4_9_a_revision_is_a_draft_and_sending_it_supersedes_the_source() {
    let app = crate::harness::start().await;
    let draft = create(&app, quote_body(&app, vec![line("Pose", 4500)])).await;
    let source_id = id_of(&draft);

    let early = reqwest::Client::new()
        .post(format!(
            "{}/api/v1/quotes/{source_id}/revision",
            app.base_url
        ))
        .bearer_auth(&app.token)
        .send()
        .await
        .expect("the api answers the revision call");
    assert_eq!(early.status(), 409);

    send(&app, &source_id).await;
    let response = reqwest::Client::new()
        .post(format!(
            "{}/api/v1/quotes/{source_id}/revision",
            app.base_url
        ))
        .bearer_auth(&app.token)
        .send()
        .await
        .expect("the api answers the revision call");
    assert_eq!(response.status(), 201);
    let revision: Value = response.json().await.unwrap();
    let revision = &revision["data"];
    let revision_id = id_of(revision);

    assert_eq!(revision["status"], json!("DRAFT"));
    assert_eq!(revision["reference"], json!(null));
    assert_eq!(revision["valid_until"], json!(null));
    assert_eq!(revision["revision_of_id"], json!(source_id));
    assert_eq!(revision["lines"].as_array().unwrap().len(), 1);
    let source = get(&app, &source_id).await;
    assert_eq!(source["status"], json!("SENT"));
    assert_eq!(source["superseded_by_id"], json!(revision_id));
    assert_ne!(source["lines"][0]["id"], revision["lines"][0]["id"]);

    send(&app, &revision_id).await;

    assert_eq!(get(&app, &source_id).await["status"], json!("SUPERSEDED"));

    app.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn spec_crm_4_2_sending_refuses_a_customer_without_billing_address_naming_the_field() {
    let app = crate::harness::start().await;
    let customer_id = Uuid::now_v7();
    let context_id = Uuid::now_v7();
    sqlx::query("INSERT INTO customers (id, org_id, name) VALUES ($1, $2, 'Sans adresse')")
        .bind(customer_id)
        .bind(app.organization_id)
        .execute(&app.pool)
        .await
        .unwrap();
    sqlx::query(
        "INSERT INTO customer_contexts (id, customer_id, label) VALUES ($1, $2, 'Chantier')",
    )
    .bind(context_id)
    .bind(customer_id)
    .execute(&app.pool)
    .await
    .unwrap();
    let created = create(
        &app,
        json!({
            "title": "Sans adresse",
            "customer_id": customer_id,
            "customer_context_id": context_id,
            "lines": [line("Pose", 4500)],
        }),
    )
    .await;
    let quote_id = id_of(&created);

    let (status, body) = set_status(&app, &quote_id, json!({ "status": "SENT" })).await;

    assert_eq!(status, 422);
    assert_eq!(body["code"], json!("E_UNPROCESSABLE_ENTITY"));
    assert_eq!(body["details"]["field"], json!("billing_address"));
    let draft = get(&app, &quote_id).await;
    assert_eq!(draft["status"], json!("DRAFT"));
    assert_eq!(draft["reference"], json!(null));

    app.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn a_rule_refused_on_create_answers_422_naming_the_field() {
    let app = crate::harness::start().await;
    let other_org = Uuid::now_v7();
    let other_owner = Uuid::now_v7();
    let other_member = Uuid::now_v7();
    sqlx::query(
        "INSERT INTO users (id, email, username, display_name, sub) VALUES ($1, $2, $3, 'Other', $4)",
    )
    .bind(other_owner)
    .bind(format!("other-{other_owner}@example.com"))
    .bind(format!("other-{other_owner}"))
    .bind(format!("sub-{other_owner}"))
    .execute(&app.pool)
    .await
    .unwrap();
    sqlx::query(
        "INSERT INTO organizations (id, name, slug, owner_id) VALUES ($1, 'Other', $2, $3)",
    )
    .bind(other_org)
    .bind(format!("other-{other_org}"))
    .bind(other_owner)
    .execute(&app.pool)
    .await
    .unwrap();
    sqlx::query("INSERT INTO organization_members (id, organization_id, last_name) VALUES ($1, $2, 'Claire')")
        .bind(other_member)
        .bind(other_org)
        .execute(&app.pool)
        .await
        .unwrap();

    for (field, patch) in [
        ("deposit_percent", json!({ "deposit_percent": "101" })),
        (
            "owner_member_id",
            json!({ "owner_member_id": other_member }),
        ),
        (
            "billing_contact_id",
            json!({ "billing_contact_id": Uuid::now_v7() }),
        ),
    ] {
        let mut body = quote_body(&app, vec![line("Pose", 4500)]);
        for (key, value) in patch.as_object().unwrap() {
            body[key] = value.clone();
        }

        let response = post_quote(&app, body).await;

        assert_eq!(response.status(), 422, "{field}");
        let answer: Value = response.json().await.unwrap();
        assert_eq!(answer["code"], json!("E_UNPROCESSABLE_ENTITY"), "{field}");
        assert_eq!(answer["details"]["field"], json!(field));
    }

    let mut discounted = quote_body(&app, vec![line("Pose", 4500)]);
    discounted["lines"][0]["discount_percent"] = json!("100.5");
    let response = post_quote(&app, discounted).await;
    assert_eq!(response.status(), 422);
    assert_eq!(
        response.json::<Value>().await.unwrap()["details"]["field"],
        json!("discount_percent")
    );

    for statement in [
        "DELETE FROM organization_members WHERE organization_id = $1",
        "DELETE FROM organizations WHERE id = $1",
    ] {
        sqlx::query(statement)
            .bind(other_org)
            .execute(&app.pool)
            .await
            .unwrap();
    }
    sqlx::query("DELETE FROM users WHERE id = $1")
        .bind(other_owner)
        .execute(&app.pool)
        .await
        .unwrap();
    app.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn spec_crm_4_14_changing_the_customer_of_a_quote_with_lines_answers_422() {
    let app = crate::harness::start().await;
    let created = create(&app, quote_body(&app, vec![line("Pose", 4500)])).await;
    let other = Uuid::now_v7();
    let other_context = Uuid::now_v7();
    sqlx::query("INSERT INTO customers (id, org_id, name) VALUES ($1, $2, 'Autre client')")
        .bind(other)
        .bind(app.organization_id)
        .execute(&app.pool)
        .await
        .unwrap();
    sqlx::query("INSERT INTO customer_contexts (id, customer_id, label) VALUES ($1, $2, 'Site')")
        .bind(other_context)
        .bind(other)
        .execute(&app.pool)
        .await
        .unwrap();

    let response = reqwest::Client::new()
        .patch(format!(
            "{}/api/v1/quotes/{}",
            app.base_url,
            id_of(&created)
        ))
        .bearer_auth(&app.token)
        .json(&json!({
            "title": "Kitchen renovation",
            "customer_id": other,
            "customer_context_id": other_context,
            "status": "DRAFT",
            "lines": [line("Pose", 4500)],
        }))
        .send()
        .await
        .expect("the api answers the update call");

    assert_eq!(response.status(), 422);
    assert_eq!(
        response.json::<Value>().await.unwrap()["details"]["field"],
        json!("customer_id")
    );
    assert_eq!(
        get(&app, &id_of(&created)).await["customer_id"],
        json!(app.customer_id)
    );

    app.cleanup().await;
}
