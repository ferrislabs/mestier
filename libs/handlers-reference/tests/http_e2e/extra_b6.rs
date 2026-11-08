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

async fn get(app: &App, token: &str, path: &str) -> (StatusCode, Value) {
    call(app, token, reqwest::Method::GET, path, None).await
}

async fn post(app: &App, token: &str, path: &str, body: Option<Value>) -> (StatusCode, Value) {
    call(app, token, reqwest::Method::POST, path, body).await
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

fn surcharges_path(app: &App) -> String {
    format!("/organizations/{}/surcharges", app.organization_id)
}

fn extras_path(app: &App) -> String {
    format!("/organizations/{}/extra-periods", app.organization_id)
}

async fn surcharges(app: &App) -> Vec<Value> {
    let (status, listed) = get(app, &app.token, &surcharges_path(app)).await;
    assert_eq!(status, StatusCode::OK, "{listed}");
    listed["data"].as_array().expect("a list").clone()
}

fn extra_body(member: Uuid, surcharge: Option<&str>) -> Value {
    json!({
        "member_id": member,
        "date": "2026-11-14",
        "start": "08:00",
        "end": "12:00",
        "reason": "Rattrapage du chantier",
        "surcharge_id": surcharge,
    })
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn spec_rh_6_1_the_first_read_creates_the_default_surcharges_once() {
    let app = harness::start().await;

    for _ in 0..2 {
        let (status, listed) = get(&app, &app.restricted_token, &surcharges_path(&app)).await;
        assert_eq!(status, StatusCode::OK, "{listed}");
        let rows = listed["data"].as_array().expect("a list");
        let seen: Vec<(&str, i64, &str)> = rows
            .iter()
            .map(|row| {
                (
                    row["name"].as_str().unwrap(),
                    row["rate_bp"].as_i64().unwrap(),
                    row["kind"].as_str().unwrap(),
                )
            })
            .collect();
        assert_eq!(
            seen,
            vec![
                ("Heures supplémentaires", 2500, "OVERTIME"),
                ("Heures supplémentaires au-delà de 8 h", 5000, "OVERTIME"),
                ("Travail du dimanche", 10000, "SUNDAY"),
                ("Jour férié travaillé", 10000, "HOLIDAY"),
                ("Travail de nuit", 2500, "NIGHT"),
            ]
        );
    }

    app.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn surcharges_are_written_with_reference_manage_and_validated() {
    let app = harness::start().await;

    let body = json!({ "name": "Astreinte", "rate_bp": 3000, "kind": "OTHER" });
    let (status, _) = post(
        &app,
        &app.restricted_token,
        &surcharges_path(&app),
        Some(body.clone()),
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN);

    for (bad, field) in [
        (
            json!({ "name": " ", "rate_bp": 3000, "kind": "OTHER" }),
            "name",
        ),
        (
            json!({ "name": "A", "rate_bp": -1, "kind": "OTHER" }),
            "rate_bp",
        ),
        (
            json!({ "name": "A", "rate_bp": 20001, "kind": "OTHER" }),
            "rate_bp",
        ),
    ] {
        let (status, refused) = post(&app, &app.token, &surcharges_path(&app), Some(bad)).await;
        assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{refused}");
        assert_eq!(refused["details"]["field"], json!(field));
    }

    let (status, created) = post(&app, &app.token, &surcharges_path(&app), Some(body)).await;
    assert_eq!(status, StatusCode::CREATED, "{created}");
    let id = created["data"]["id"].as_str().expect("an id").to_owned();
    let path = format!("{}/{id}", surcharges_path(&app));

    let (status, updated) = call(
        &app,
        &app.token,
        reqwest::Method::PUT,
        &path,
        Some(json!({ "name": "Astreinte lourde", "rate_bp": 4500, "kind": "NIGHT" })),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{updated}");
    assert_eq!(updated["data"]["rate_bp"], json!(4500));
    assert_eq!(updated["data"]["kind"], json!("NIGHT"));

    let (status, _) = call(
        &app,
        &app.token,
        reqwest::Method::PUT,
        &format!("{}/{}", surcharges_path(&app), Uuid::now_v7()),
        Some(json!({ "name": "Fantôme", "rate_bp": 100, "kind": "OTHER" })),
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);

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
    let (status, restored) = post(&app, &app.token, &format!("{path}/restore"), None).await;
    assert_eq!(status, StatusCode::OK, "{restored}");
    assert!(restored["data"]["archived_at"].is_null());
    assert_eq!(surcharges(&app).await.len(), 6);

    app.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn spec_rh_6_2_a_period_copies_the_rate_and_a_later_change_leaves_it_unchanged() {
    let app = harness::start().await;
    let member = colleague(&app).await;
    let sunday = surcharges(&app).await[2]["id"].as_str().unwrap().to_owned();

    let (status, created) = post(
        &app,
        &app.token,
        &extras_path(&app),
        Some(extra_body(member, Some(&sunday))),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "{created}");
    assert_eq!(created["data"]["rate_bp"], json!(10000));
    assert_eq!(created["data"]["status"], json!("PENDING"));
    assert_eq!(created["data"]["surcharge_id"], json!(sunday));
    assert_eq!(created["data"]["start"], json!("08:00"));
    assert!(created["data"]["created_by_member_id"].is_string());

    let (status, _) = call(
        &app,
        &app.token,
        reqwest::Method::PUT,
        &format!("{}/{sunday}", surcharges_path(&app)),
        Some(json!({ "name": "Dimanche", "rate_bp": 5000, "kind": "SUNDAY" })),
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    let (status, listed) = get(&app, &app.token, &extras_path(&app)).await;
    assert_eq!(status, StatusCode::OK, "{listed}");
    assert_eq!(listed["data"][0]["rate_bp"], json!(10000));

    let mut free = extra_body(member, None);
    free["rate_bp"] = json!(3000);
    free["approve"] = json!(true);
    let (status, free) = post(&app, &app.token, &extras_path(&app), Some(free)).await;
    assert_eq!(status, StatusCode::CREATED, "{free}");
    assert_eq!(free["data"]["rate_bp"], json!(3000));
    assert_eq!(free["data"]["status"], json!("APPROVED"));
    assert!(free["data"]["surcharge_id"].is_null());

    app.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn spec_rh_6_3_approve_and_reject_follow_the_absence_rules() {
    let app = harness::start().await;
    let member = colleague(&app).await;
    let mut ids = Vec::new();
    for _ in 0..3 {
        let (status, created) = post(
            &app,
            &app.token,
            &extras_path(&app),
            Some(extra_body(member, None)),
        )
        .await;
        assert_eq!(status, StatusCode::CREATED, "{created}");
        ids.push(format!(
            "{}/{}",
            extras_path(&app),
            created["data"]["id"].as_str().unwrap()
        ));
    }

    let (status, _) = post(
        &app,
        &app.restricted_token,
        &format!("{}/approve", ids[0]),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    let (status, approved) = post(&app, &app.token, &format!("{}/approve", ids[0]), None).await;
    assert_eq!(status, StatusCode::OK, "{approved}");
    assert_eq!(approved["data"]["status"], json!("APPROVED"));
    assert!(approved["data"]["decided_by_member_id"].is_string());
    assert!(approved["data"]["decided_at"].is_string());

    let (status, again) = post(&app, &app.token, &format!("{}/approve", ids[0]), None).await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{again}");
    assert_eq!(again["details"]["field"], json!("status"));

    let (status, blank) = post(
        &app,
        &app.token,
        &format!("{}/reject", ids[1]),
        Some(json!({ "comment": "  " })),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{blank}");
    assert_eq!(blank["details"]["field"], json!("comment"));
    let (status, rejected) = post(
        &app,
        &app.token,
        &format!("{}/reject", ids[1]),
        Some(json!({ "comment": "Pas de commande" })),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{rejected}");
    assert_eq!(rejected["data"]["status"], json!("REJECTED"));
    assert_eq!(
        rejected["data"]["decision_comment"],
        json!("Pas de commande")
    );
    let (status, _) = post(&app, &app.token, &format!("{}/cancel", ids[1]), None).await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);

    let (status, _) = post(
        &app,
        &app.restricted_token,
        &format!("{}/cancel", ids[2]),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    let (status, cancelled) = post(&app, &app.token, &format!("{}/cancel", ids[2]), None).await;
    assert_eq!(status, StatusCode::OK, "{cancelled}");
    assert_eq!(cancelled["data"]["status"], json!("CANCELLED"));

    let missing = format!("{}/{}/approve", extras_path(&app), Uuid::now_v7());
    let (status, _) = post(&app, &app.token, &missing, None).await;
    assert_eq!(status, StatusCode::NOT_FOUND);

    app.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn spec_rh_6_4_an_end_not_after_the_start_is_refused() {
    let app = harness::start().await;
    let member = colleague(&app).await;

    let (status, _) = post(
        &app,
        &app.restricted_token,
        &extras_path(&app),
        Some(extra_body(member, None)),
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN);

    let with = |patch: Value| {
        let mut body = extra_body(member, None);
        for (key, value) in patch.as_object().unwrap() {
            body[key] = value.clone();
        }
        body
    };
    for (body, field) in [
        (with(json!({ "end": "08:00" })), "end"),
        (with(json!({ "end": "07:00" })), "end"),
        (with(json!({ "start": "8h" })), "start"),
        (
            with(json!({ "lunch": { "start": "07:00", "end": "09:00" } })),
            "lunch",
        ),
        (with(json!({ "reason": " " })), "reason"),
        (with(json!({ "rate_bp": 20001 })), "rate_bp"),
        (with(json!({ "member_id": Uuid::now_v7() })), "member_id"),
        (
            with(json!({ "surcharge_id": Uuid::now_v7() })),
            "surcharge_id",
        ),
    ] {
        let (status, refused) = post(&app, &app.token, &extras_path(&app), Some(body)).await;
        assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{refused}");
        assert_eq!(refused["details"]["field"], json!(field), "{refused}");
    }
    let (_, listed) = get(&app, &app.token, &extras_path(&app)).await;
    assert!(listed["data"].as_array().unwrap().is_empty());

    app.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn an_archived_surcharge_is_refused_for_new_periods_and_stays_on_past_ones() {
    let app = harness::start().await;
    let member = colleague(&app).await;
    let night = surcharges(&app).await[4]["id"].as_str().unwrap().to_owned();
    let (status, past) = post(
        &app,
        &app.token,
        &extras_path(&app),
        Some(extra_body(member, Some(&night))),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "{past}");

    let (status, _) = post(
        &app,
        &app.token,
        &format!("{}/{night}/archive", surcharges_path(&app)),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let (status, refused) = post(
        &app,
        &app.token,
        &extras_path(&app),
        Some(extra_body(member, Some(&night))),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{refused}");
    assert_eq!(refused["details"]["field"], json!("surcharge_id"));

    let (_, listed) = get(&app, &app.token, &extras_path(&app)).await;
    assert_eq!(listed["data"].as_array().unwrap().len(), 1);
    assert_eq!(listed["data"][0]["surcharge_id"], json!(night));

    app.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn extra_periods_are_filtered_paginated_and_read_with_planning_management() {
    let app = harness::start().await;
    let member = colleague(&app).await;
    let other = colleague(&app).await;
    for (who, date, approve) in [
        (member, "2026-11-14", false),
        (member, "2026-10-03", true),
        (other, "2026-11-02", false),
    ] {
        let mut body = extra_body(who, None);
        body["date"] = json!(date);
        body["approve"] = json!(approve);
        let (status, created) = post(&app, &app.token, &extras_path(&app), Some(body)).await;
        assert_eq!(status, StatusCode::CREATED, "{created}");
    }

    let count = |query: String| {
        let app = &app;
        async move {
            let path = format!("{}?{query}", extras_path(app));
            let (status, listed) = get(app, &app.token, &path).await;
            assert_eq!(status, StatusCode::OK, "{listed}");
            (
                listed["data"].as_array().unwrap().len(),
                listed["pagination"]["total"].as_u64(),
            )
        }
    };
    assert_eq!(count(String::new()).await, (3, Some(3)));
    assert_eq!(count(format!("member_id={member}")).await, (2, Some(2)));
    assert_eq!(count("status=PENDING".to_owned()).await, (2, Some(2)));
    assert_eq!(
        count("from=2026-11-01&to=2026-11-30".to_owned()).await,
        (2, Some(2))
    );
    assert_eq!(count("per_page=1&page=2".to_owned()).await, (1, Some(3)));

    let (_, first_page) = get(
        &app,
        &app.token,
        &format!("{}?per_page=1", extras_path(&app)),
    )
    .await;
    assert_eq!(first_page["data"][0]["date"], json!("2026-11-14"));

    let (status, _) = get(&app, &app.restricted_token, &extras_path(&app)).await;
    assert_eq!(status, StatusCode::FORBIDDEN);

    app.cleanup().await;
}
