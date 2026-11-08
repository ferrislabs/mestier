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

async fn own_member(app: &App) -> Uuid {
    sqlx::query_scalar("SELECT member_id FROM employees WHERE id = $1")
        .bind(app.employee_id)
        .fetch_one(&app.pool)
        .await
        .expect("the fixture employee profiles a member")
}

async fn other_member(app: &App) -> Uuid {
    sqlx::query_scalar("SELECT member_id FROM employees WHERE id = $1")
        .bind(app.other_employee_id)
        .fetch_one(&app.pool)
        .await
        .expect("the other employee profiles a member")
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

fn path(app: &App, member: Uuid, tail: &str) -> String {
    format!(
        "/organizations/{}/members/{member}/{tail}",
        app.organization_id
    )
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn spec_rh_1_1_a_member_without_profile_reads_defaults_and_a_put_creates_it() {
    let app = harness::start().await;
    let member = colleague(&app).await;
    let profile = path(&app, member, "profile");

    let (status, read) = call(&app, &app.token, reqwest::Method::GET, &profile, None).await;
    assert_eq!(status, StatusCode::OK, "{read}");
    assert_eq!(read["data"]["work_email"], Value::Null);
    assert_eq!(read["data"]["address"]["country"], json!("France"));

    let body = json!({
        "work_email": "a@b.fr",
        "mobile": "06 12 34 56 78",
        "address": { "street": "1 rue X", "postal_code": "75011", "city": "Paris" },
        "job_title": "Chef de chantier",
        "work_permit_end_date": "2027-01-31",
    });
    let (status, put) = call(&app, &app.token, reqwest::Method::PUT, &profile, Some(body)).await;
    assert_eq!(status, StatusCode::OK, "{put}");
    assert_eq!(put["data"]["work_permit_end_date"], json!("2027-01-31"));

    let (_, read) = call(&app, &app.token, reqwest::Method::GET, &profile, None).await;
    assert_eq!(read["data"]["job_title"], json!("Chef de chantier"));
    assert_eq!(read["data"]["address"]["postal_code"], json!("75011"));

    let (status, refused) = call(
        &app,
        &app.token,
        reqwest::Method::PUT,
        &profile,
        Some(json!({ "work_email": "nope" })),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{refused}");
    assert_eq!(refused["details"]["field"], json!("work_email"));

    let (status, _) = call(
        &app,
        &app.restricted_token,
        reqwest::Method::PUT,
        &profile,
        Some(json!({})),
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN);

    app.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn spec_rh_1_2_a_foreign_self_or_cyclic_manager_is_refused() {
    let app = harness::start().await;
    let a = own_member(&app).await;
    let b = colleague(&app).await;
    let foreign = other_member(&app).await;
    let put = |member: Uuid, manager: Uuid| {
        let app = &app;
        async move {
            call(
                app,
                &app.token,
                reqwest::Method::PUT,
                &path(app, member, "profile"),
                Some(json!({ "manager_member_id": manager })),
            )
            .await
        }
    };

    let (status, body) = put(a, a).await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{body}");
    let (status, body) = put(a, foreign).await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{body}");
    let (status, body) = put(a, b).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let (status, body) = put(b, a).await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{body}");
    assert_eq!(body["code"], json!("E_MEMBER_PROFILE_MANAGER_CYCLE"));

    app.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn spec_rh_1_3_archiving_sets_archived_at_and_restoring_clears_it() {
    let app = harness::start().await;
    let member = colleague(&app).await;

    let (status, archived) = call(
        &app,
        &app.token,
        reqwest::Method::POST,
        &path(&app, member, "archive"),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{archived}");
    assert!(archived["data"]["archived_at"].is_string());

    let (status, restored) = call(
        &app,
        &app.token,
        reqwest::Method::POST,
        &path(&app, member, "restore"),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{restored}");
    assert_eq!(restored["data"]["archived_at"], Value::Null);

    app.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn spec_rh_1_4_a_member_of_another_organization_answers_not_found() {
    let app = harness::start().await;
    let foreign = other_member(&app).await;

    for (method, tail) in [
        (reqwest::Method::GET, "profile"),
        (reqwest::Method::POST, "archive"),
    ] {
        let (status, body) = call(&app, &app.token, method, &path(&app, foreign, tail), None).await;
        assert_eq!(status, StatusCode::NOT_FOUND, "{tail}: {body}");
    }

    app.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn spec_rh_1_5_the_list_returns_the_profiles_of_the_organization_only() {
    let app = harness::start().await;
    let member = colleague(&app).await;
    let foreign = other_member(&app).await;
    let (status, _) = call(
        &app,
        &app.token,
        reqwest::Method::PUT,
        &path(&app, member, "profile"),
        Some(json!({ "job_title": "Chef" })),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    sqlx::query(
        "INSERT INTO member_hr_profiles (member_id, org_id) SELECT id, organization_id FROM organization_members WHERE id = $1",
    )
    .bind(foreign)
    .execute(&app.pool)
    .await
    .expect("seed a foreign profile");

    let list = format!("/organizations/{}/member-profiles", app.organization_id);
    let (status, body) = call(&app, &app.token, reqwest::Method::GET, &list, None).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let ids: Vec<&str> = body["data"]
        .as_array()
        .expect("a list")
        .iter()
        .map(|profile| profile["member_id"].as_str().unwrap_or_default())
        .collect();
    assert!(ids.contains(&member.to_string().as_str()), "{body}");
    assert!(!ids.contains(&foreign.to_string().as_str()), "{body}");

    app.cleanup().await;
}
