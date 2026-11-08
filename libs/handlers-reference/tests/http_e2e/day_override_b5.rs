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

async fn colleague(app: &App, organization_id: Uuid) -> Uuid {
    let id = Uuid::now_v7();
    sqlx::query("INSERT INTO organization_members (id, organization_id, last_name) VALUES ($1, $2, 'Collegue')")
        .bind(id)
        .bind(organization_id)
        .execute(&app.pool)
        .await
        .expect("seed a colleague");
    id
}

fn override_path(app: &App, member: Uuid, date: &str) -> String {
    format!(
        "/organizations/{}/members/{member}/day-overrides/{date}",
        app.organization_id
    )
}

fn list_path(app: &App, from: &str, to: &str) -> String {
    format!(
        "/organizations/{}/day-overrides?from={from}&to={to}",
        app.organization_id
    )
}

fn feed_path(app: &App, from: &str, to: &str) -> String {
    format!(
        "/organizations/{}/hr/planning-feed?from={from}&to={to}",
        app.organization_id
    )
}

fn work(start: &str, end: &str) -> Value {
    json!({ "kind": "WORK", "start": start, "end": end, "note": "Chantier" })
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn spec_rh_5_1_an_override_is_upserted_per_member_and_day() {
    let app = harness::start().await;
    let member = colleague(&app, app.organization_id).await;
    let path = override_path(&app, member, "2026-11-10");

    let (status, first) = call(
        &app,
        &app.token,
        reqwest::Method::PUT,
        &path,
        Some(json!({
            "kind": "WORK", "start": "08:00", "end": "17:00",
            "lunch": { "start": "12:00", "end": "13:00" },
        })),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{first}");
    assert_eq!(
        first["data"]["lunch"],
        json!({ "start": "12:00", "end": "13:00" })
    );
    let (status, second) = call(
        &app,
        &app.token,
        reqwest::Method::PUT,
        &path,
        Some(json!({ "kind": "OFF", "note": "Pont" })),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{second}");

    let (status, listed) = call(
        &app,
        &app.token,
        reqwest::Method::GET,
        &list_path(&app, "2026-11-01", "2026-11-30"),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{listed}");
    let rows = listed["data"].as_array().expect("a list");
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0]["id"], first["data"]["id"]);
    assert_eq!(rows[0]["kind"], json!("OFF"));
    assert_eq!(rows[0]["start"], Value::Null);
    assert_eq!(rows[0]["note"], json!("Pont"));

    let (status, _) = call(&app, &app.token, reqwest::Method::DELETE, &path, None).await;
    assert_eq!(status, StatusCode::NO_CONTENT);
    let (status, _) = call(&app, &app.token, reqwest::Method::DELETE, &path, None).await;
    assert_eq!(status, StatusCode::NOT_FOUND);

    app.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn spec_rh_5_2_a_work_override_with_end_before_start_is_refused() {
    let app = harness::start().await;
    let member = colleague(&app, app.organization_id).await;
    let path = override_path(&app, member, "2026-11-10");

    for (body, field) in [
        (work("17:00", "08:00"), "end"),
        (json!({ "kind": "WORK", "start": "08:00" }), "end"),
        (
            json!({ "kind": "WORK", "start": "8h", "end": "17:00" }),
            "start",
        ),
        (
            json!({ "kind": "OFF", "start": "08:00", "end": "17:00" }),
            "kind",
        ),
        (
            json!({ "kind": "WORK", "start": "08:00", "end": "12:00", "lunch": { "start": "11:00", "end": "13:00" } }),
            "lunch",
        ),
    ] {
        let (status, refused) =
            call(&app, &app.token, reqwest::Method::PUT, &path, Some(body)).await;
        assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{refused}");
        assert_eq!(refused["details"]["field"], json!(field), "{refused}");
    }
    let (_, listed) = call(
        &app,
        &app.token,
        reqwest::Method::GET,
        &list_path(&app, "2026-11-01", "2026-11-30"),
        None,
    )
    .await;
    assert_eq!(listed["data"], json!([]));

    app.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn overrides_need_planning_management_and_stay_inside_the_organization() {
    let app = harness::start().await;
    let member = colleague(&app, app.organization_id).await;
    let stranger = colleague(&app, app.other_organization_id().await).await;

    let (status, _) = call(
        &app,
        &app.restricted_token,
        reqwest::Method::PUT,
        &override_path(&app, member, "2026-11-10"),
        Some(work("08:00", "17:00")),
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    for path in [
        list_path(&app, "2026-11-01", "2026-11-30"),
        feed_path(&app, "2026-11-01", "2026-11-30"),
    ] {
        let (status, _) = call(
            &app,
            &app.restricted_token,
            reqwest::Method::GET,
            &path,
            None,
        )
        .await;
        assert_eq!(status, StatusCode::FORBIDDEN);
    }
    let (status, _) = call(
        &app,
        &app.token,
        reqwest::Method::PUT,
        &override_path(&app, stranger, "2026-11-10"),
        Some(work("08:00", "17:00")),
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);

    app.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn spec_rh_5_3_the_planning_feed_returns_only_the_requested_period() {
    let app = harness::start().await;
    let member = colleague(&app, app.organization_id).await;
    for date in ["2026-10-28", "2026-11-10", "2026-12-20"] {
        let (status, body) = call(
            &app,
            &app.token,
            reqwest::Method::PUT,
            &override_path(&app, member, date),
            Some(work("08:00", "17:00")),
        )
        .await;
        assert_eq!(status, StatusCode::OK, "{body}");
    }
    for (start, end) in [
        ("2026-10-20", "2026-10-25"),
        ("2026-11-12", "2026-11-13"),
        ("2026-12-01", "2026-12-03"),
    ] {
        let (status, body) = call(
            &app,
            &app.token,
            reqwest::Method::POST,
            &format!("/organizations/{}/absences", app.organization_id),
            Some(json!({
                "member_id": member, "kind": "LEAVE",
                "start_date": start, "start_part": "MORNING",
                "end_date": end, "end_part": "AFTERNOON",
            })),
        )
        .await;
        assert_eq!(status, StatusCode::CREATED, "{body}");
    }
    let (status, contract) = call(
        &app,
        &app.token,
        reqwest::Method::POST,
        &format!(
            "/organizations/{}/members/{member}/contracts",
            app.organization_id
        ),
        Some(json!({
            "type": "CDI", "start_date": "2026-01-01",
            "schedule": { "kind": "variable", "default_day": { "start": "08:00", "end": "16:00" } },
        })),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "{contract}");

    let (status, feed) = call(
        &app,
        &app.token,
        reqwest::Method::GET,
        &feed_path(&app, "2026-11-01", "2026-11-30"),
        None,
    )
    .await;

    assert_eq!(status, StatusCode::OK, "{feed}");
    let data = &feed["data"];
    let dates = |key: &str, field: &str| -> Vec<Value> {
        data[key]
            .as_array()
            .expect("a list")
            .iter()
            .map(|row| row[field].clone())
            .collect()
    };
    assert_eq!(dates("overrides", "date"), vec![json!("2026-11-10")]);
    assert_eq!(dates("absences", "start_date"), vec![json!("2026-11-12")]);
    assert_eq!(data["contracts"].as_array().expect("a list").len(), 1);
    assert!(
        data["members"]
            .as_array()
            .expect("a list")
            .iter()
            .any(|row| row["id"] == json!(member) && row["archived"] == json!(false))
    );
    assert!(!data["absence_types"].as_array().expect("a list").is_empty());

    app.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn the_planning_feed_refuses_a_period_over_62_days_or_inverted() {
    let app = harness::start().await;

    let (status, refused) = call(
        &app,
        &app.token,
        reqwest::Method::GET,
        &feed_path(&app, "2026-11-01", "2027-01-02"),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{refused}");
    let (status, _) = call(
        &app,
        &app.token,
        reqwest::Method::GET,
        &feed_path(&app, "2026-11-01", "2026-12-31"),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let (status, _) = call(
        &app,
        &app.token,
        reqwest::Method::GET,
        &feed_path(&app, "2026-11-10", "2026-11-09"),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);

    app.cleanup().await;
}
