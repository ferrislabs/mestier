use reqwest::StatusCode;
use serde_json::{Value, json};
use uuid::Uuid;

use crate::harness::{self, App};

async fn call(
    app: &App,
    token: &str,
    method: reqwest::Method,
    path: &str,
    body: Option<Value>,
) -> (StatusCode, Value) {
    let request = reqwest::Client::new()
        .request(method, app.url(path))
        .bearer_auth(token);
    let request = match body {
        Some(body) => request.json(&body),
        None => request,
    };
    let response = request.send().await.expect("the api answers");
    let status = response.status();
    let text = response.text().await.unwrap_or_default();

    (status, serde_json::from_str(&text).unwrap_or(Value::Null))
}

async fn colleague(app: &App) -> Uuid {
    let id = Uuid::now_v7();
    sqlx::query("INSERT INTO organization_members (id, organization_id, last_name) VALUES ($1, $2, 'Collegue')")
        .bind(id)
        .bind(app.organization_id)
        .execute(&app.pool)
        .await
        .expect("seed a colleague");
    id
}

fn accounts_path(app: &App) -> String {
    format!("/organizations/{}/leave-accounts", app.organization_id)
}

fn adjustments_path(app: &App) -> String {
    format!("/organizations/{}/leave-adjustments", app.organization_id)
}

fn types_path(app: &App) -> String {
    format!("/organizations/{}/absence-types", app.organization_id)
}

async fn get(app: &App, token: &str, path: &str) -> (StatusCode, Value) {
    call(app, token, reqwest::Method::GET, path, None).await
}

async fn post(app: &App, token: &str, path: &str, body: Option<Value>) -> (StatusCode, Value) {
    call(app, token, reqwest::Method::POST, path, body).await
}

async fn accounts(app: &App) -> Vec<Value> {
    let (status, listed) = get(app, &app.token, &accounts_path(app)).await;
    assert_eq!(status, StatusCode::OK, "{listed}");
    listed["data"].as_array().expect("a list").clone()
}

fn account_body(name: &str) -> Value {
    json!({
        "name": name,
        "acquisition": { "kind": "MONTHLY", "per_month": 0.25 },
        "period_start_month": 9,
        "carry_over_max": 2.5,
    })
}

fn adjustment_body(member: Uuid, account: &str, days: f64, reason: &str) -> Value {
    json!({
        "member_id": member,
        "account_id": account,
        "date": "2026-07-01",
        "days": days,
        "kind": "ADJUSTMENT",
        "reason": reason,
    })
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn spec_rh_4_1_the_first_read_creates_the_default_accounts_once() {
    let app = harness::start().await;

    for _ in 0..2 {
        let (status, listed) = get(&app, &app.restricted_token, &accounts_path(&app)).await;
        assert_eq!(status, StatusCode::OK, "{listed}");
        let rows = listed["data"].as_array().expect("a list");
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0]["name"], json!("Congés payés"));
        assert_eq!(
            rows[0]["acquisition"],
            json!({ "kind": "MONTHLY", "per_month": 2.08 })
        );
        assert_eq!(rows[0]["period_start_month"], json!(6));
        assert_eq!(rows[0]["carry_over_max"], json!(5.0));
        assert_eq!(rows[1]["name"], json!("RTT"));
        assert_eq!(
            rows[1]["acquisition"],
            json!({ "kind": "ANNUAL", "days": 10.0 })
        );
    }

    let (_, types) = get(&app, &app.token, &types_path(&app)).await;
    let cp = accounts(&app).await[0]["id"].clone();
    assert_eq!(types["data"][0]["name"], json!("Congés payés"));
    assert_eq!(types["data"][0]["account_id"], cp);

    app.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn accounts_are_written_with_reference_manage_and_validated() {
    let app = harness::start().await;

    let (status, _) = post(
        &app,
        &app.restricted_token,
        &accounts_path(&app),
        Some(account_body("Interdit")),
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN);

    for (body, field) in [
        (account_body(" "), "name"),
        (
            json!({ "name": "A", "acquisition": { "kind": "ANNUAL", "days": 0 }, "period_start_month": 1, "carry_over_max": 0 }),
            "acquisition",
        ),
        (
            json!({ "name": "A", "acquisition": { "kind": "ANNUAL", "days": 1.234 }, "period_start_month": 1, "carry_over_max": 0 }),
            "acquisition",
        ),
        (
            json!({ "name": "A", "acquisition": { "kind": "ANNUAL", "days": 1 }, "period_start_month": 13, "carry_over_max": 0 }),
            "period_start_month",
        ),
        (
            json!({ "name": "A", "acquisition": { "kind": "ANNUAL", "days": 1 }, "period_start_month": 1, "carry_over_max": -1 }),
            "carry_over_max",
        ),
    ] {
        let (status, refused) = post(&app, &app.token, &accounts_path(&app), Some(body)).await;
        assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{refused}");
        assert_eq!(refused["details"]["field"], json!(field));
    }

    let (status, created) = post(
        &app,
        &app.token,
        &accounts_path(&app),
        Some(account_body("Ancienneté")),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "{created}");
    let id = created["data"]["id"].as_str().expect("an id").to_owned();
    assert_eq!(created["data"]["carry_over_max"], json!(2.5));

    let (status, updated) = call(
        &app,
        &app.token,
        reqwest::Method::PUT,
        &format!("{}/{id}", accounts_path(&app)),
        Some(json!({ "name": "Ancienneté bis", "acquisition": { "kind": "ANNUAL", "days": 3 }, "period_start_month": 1, "carry_over_max": 0 })),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{updated}");
    assert_eq!(
        updated["data"]["acquisition"],
        json!({ "kind": "ANNUAL", "days": 3.0 })
    );

    let (status, _) = call(
        &app,
        &app.token,
        reqwest::Method::PUT,
        &format!("{}/{}", accounts_path(&app), Uuid::now_v7()),
        Some(account_body("Fantôme")),
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);

    app.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn spec_rh_4_2_an_adjustment_of_zero_days_or_without_a_reason_is_refused() {
    let app = harness::start().await;
    let member = colleague(&app).await;
    let cp = accounts(&app).await[0]["id"].as_str().unwrap().to_owned();

    let (status, _) = post(
        &app,
        &app.restricted_token,
        &adjustments_path(&app),
        Some(adjustment_body(member, &cp, 1.0, "Interdit")),
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN);

    for (body, field) in [
        (adjustment_body(member, &cp, 0.0, "Erreur"), "days"),
        (adjustment_body(member, &cp, 1.0, "  "), "reason"),
        (adjustment_body(member, &cp, 0.333, "Erreur"), "days"),
        (
            adjustment_body(Uuid::now_v7(), &cp, 1.0, "Erreur"),
            "member_id",
        ),
        (
            adjustment_body(member, &Uuid::now_v7().to_string(), 1.0, "Erreur"),
            "account_id",
        ),
        (
            json!({ "member_id": member, "account_id": cp, "date": "2026-07-01", "days": 6, "kind": "CARRY_OVER", "reason": "Report" }),
            "days",
        ),
    ] {
        let (status, refused) = post(&app, &app.token, &adjustments_path(&app), Some(body)).await;
        assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{refused}");
        assert_eq!(refused["details"]["field"], json!(field));
    }

    let (status, created) = post(
        &app,
        &app.token,
        &adjustments_path(&app),
        Some(adjustment_body(member, &cp, -1.5, "Erreur de saisie")),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "{created}");
    assert_eq!(created["data"]["days"], json!(-1.5));
    assert!(created["data"]["author_member_id"].is_string());

    let (status, listed) = get(&app, &app.token, &adjustments_path(&app)).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(listed["data"].as_array().unwrap().len(), 1);

    app.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn spec_rh_4_3_deleting_an_adjustment_removes_it_from_the_list() {
    let app = harness::start().await;
    let member = colleague(&app).await;
    let other = colleague(&app).await;
    let accounts = accounts(&app).await;
    let cp = accounts[0]["id"].as_str().unwrap().to_owned();
    let rtt = accounts[1]["id"].as_str().unwrap().to_owned();

    let mut ids = Vec::new();
    for (who, account) in [(member, &cp), (member, &rtt), (other, &cp)] {
        let (status, created) = post(
            &app,
            &app.token,
            &adjustments_path(&app),
            Some(adjustment_body(who, account, 1.0, "Geste")),
        )
        .await;
        assert_eq!(status, StatusCode::CREATED, "{created}");
        ids.push(created["data"]["id"].as_str().unwrap().to_owned());
    }

    let filtered = |query: String| {
        let path = format!("{}?{query}", adjustments_path(&app));
        let app = &app;
        async move {
            let (status, listed) = get(app, &app.token, &path).await;
            assert_eq!(status, StatusCode::OK, "{listed}");
            listed["data"].as_array().unwrap().len()
        }
    };
    assert_eq!(filtered(format!("member_id={member}")).await, 2);
    assert_eq!(filtered(format!("account_id={cp}")).await, 2);
    assert_eq!(
        filtered(format!("member_id={member}&account_id={rtt}")).await,
        1
    );

    let (status, _) = get(
        &app,
        &app.restricted_token,
        &format!("{}?member_id={member}", adjustments_path(&app)),
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN);

    let path = format!("{}/{}", adjustments_path(&app), ids[0]);
    let (status, _) = call(
        &app,
        &app.restricted_token,
        reqwest::Method::DELETE,
        &path,
        None,
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    let (status, _) = call(&app, &app.token, reqwest::Method::DELETE, &path, None).await;
    assert_eq!(status, StatusCode::NO_CONTENT);
    let (status, _) = call(&app, &app.token, reqwest::Method::DELETE, &path, None).await;
    assert_eq!(status, StatusCode::NOT_FOUND);

    let (_, listed) = get(&app, &app.token, &adjustments_path(&app)).await;
    let remaining: Vec<&str> = listed["data"]
        .as_array()
        .unwrap()
        .iter()
        .map(|row| row["id"].as_str().unwrap())
        .collect();
    assert_eq!(remaining.len(), 2);
    assert!(!remaining.contains(&ids[0].as_str()));

    app.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn spec_rh_4_4_an_account_with_adjustments_is_archived_not_deleted() {
    let app = harness::start().await;
    let member = colleague(&app).await;
    let cp = accounts(&app).await[0]["id"].as_str().unwrap().to_owned();
    let (status, _) = post(
        &app,
        &app.token,
        &adjustments_path(&app),
        Some(adjustment_body(member, &cp, 2.0, "Geste")),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);

    let path = format!("{}/{cp}", accounts_path(&app));
    let (status, _) = call(&app, &app.token, reqwest::Method::DELETE, &path, None).await;
    assert_eq!(status, StatusCode::METHOD_NOT_ALLOWED);

    let (status, _) = post(
        &app,
        &app.restricted_token,
        &format!("{path}/archive"),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    let (status, archived) = post(&app, &app.token, &format!("{path}/archive"), None).await;
    assert_eq!(status, StatusCode::OK, "{archived}");
    assert!(archived["data"]["archived_at"].is_string());

    let rows = accounts(&app).await;
    assert_eq!(rows.len(), 2);
    assert!(rows[0]["archived_at"].is_string());

    let (status, refused) = post(
        &app,
        &app.token,
        &adjustments_path(&app),
        Some(adjustment_body(member, &cp, 1.0, "Trop tard")),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{refused}");
    assert_eq!(refused["details"]["field"], json!("account_id"));

    let (status, refused) = post(
        &app,
        &app.token,
        &types_path(&app),
        Some(json!({ "name": "Autre", "hue": 10, "count_mode": "WORKING", "account_id": cp,
                      "paid": true, "needs_approval": true, "needs_proof": false, "kind": "LEAVE" })),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{refused}");
    assert_eq!(refused["details"]["field"], json!("account_id"));

    let (_, kept) = get(&app, &app.token, &adjustments_path(&app)).await;
    assert_eq!(kept["data"].as_array().unwrap().len(), 1);

    let (status, restored) = post(&app, &app.token, &format!("{path}/restore"), None).await;
    assert_eq!(status, StatusCode::OK, "{restored}");
    assert!(restored["data"]["archived_at"].is_null());

    app.cleanup().await;
}
