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

async fn foreign_member(app: &App) -> Uuid {
    sqlx::query_scalar("SELECT member_id FROM employees WHERE id = $1")
        .bind(app.other_employee_id)
        .fetch_one(&app.pool)
        .await
        .expect("the other employee profiles a member")
}

fn member_path(app: &App, member: Uuid, tail: &str) -> String {
    format!(
        "/organizations/{}/members/{member}/{tail}",
        app.organization_id
    )
}

fn week() -> Value {
    let open =
        json!({ "start": "08:00", "end": "17:00", "lunch": { "start": "12:00", "end": "13:00" } });
    json!([open, open, open, open, open, null, null])
}

fn contract(start: &str, end: Option<&str>, schedule: Value) -> Value {
    json!({
        "type": "CDI",
        "start_date": start,
        "end_date": end,
        "trial_end_date": null,
        "schedule": schedule,
    })
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn spec_rh_2_1_the_three_schedule_kinds_round_trip_through_the_api() {
    let app = harness::start().await;
    let member = colleague(&app).await;
    let contracts = member_path(&app, member, "contracts");
    let schedules = [
        (
            "2025-01-01",
            "2025-12-31",
            json!({ "kind": "weekly", "week": week() }),
        ),
        (
            "2026-01-01",
            "2026-12-31",
            json!({ "kind": "rotation", "anchor": "2026-01-05", "blocks": [
                { "length": 2, "unit": "weeks", "label": "Chantier", "week": week() },
                { "length": 1, "unit": "months", "label": "Repos", "week": [null, null, null, null, null, null, null] },
            ] }),
        ),
        (
            "2027-01-01",
            "2027-12-31",
            json!({ "kind": "variable", "default_day": { "start": "09:00", "end": "17:30" } }),
        ),
    ];
    for (start, end, schedule) in schedules {
        let body = contract(start, Some(end), schedule.clone());
        let (status, created) = call(
            &app,
            &app.token,
            reqwest::Method::POST,
            &contracts,
            Some(body),
        )
        .await;
        assert_eq!(status, StatusCode::CREATED, "{created}");
        assert_eq!(created["data"]["schedule"], schedule);
        assert_eq!(created["data"]["type"], json!("CDI"));
    }

    let (status, listed) = call(&app, &app.token, reqwest::Method::GET, &contracts, None).await;
    assert_eq!(status, StatusCode::OK, "{listed}");
    let rows = listed["data"].as_array().expect("a list");
    assert_eq!(rows.len(), 3);
    assert_eq!(rows[1]["schedule"]["kind"], json!("rotation"));
    assert_eq!(rows[2]["schedule"]["default_day"]["end"], json!("17:30"));

    let organization = format!("/organizations/{}/member-contracts", app.organization_id);
    let (status, all) = call(&app, &app.token, reqwest::Method::GET, &organization, None).await;
    assert_eq!(status, StatusCode::OK, "{all}");
    assert!(all["data"].as_array().expect("a list").len() >= 3);

    app.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn spec_rh_2_2_a_broken_day_is_refused_naming_it() {
    let app = harness::start().await;
    let member = colleague(&app).await;
    let contracts = member_path(&app, member, "contracts");
    let mut broken = week();
    broken[2] = json!({ "start": "17:00", "end": "08:00" });
    let (status, refused) = call(
        &app,
        &app.token,
        reqwest::Method::POST,
        &contracts,
        Some(contract(
            "2026-01-01",
            None,
            json!({ "kind": "weekly", "week": broken }),
        )),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{refused}");
    assert_eq!(refused["details"]["field"], json!("schedule.week[2]"));

    let mut lunch = week();
    lunch[4] =
        json!({ "start": "08:00", "end": "17:00", "lunch": { "start": "06:00", "end": "09:00" } });
    let (status, refused) = call(
        &app,
        &app.token,
        reqwest::Method::POST,
        &contracts,
        Some(contract(
            "2026-01-01",
            None,
            json!({ "kind": "weekly", "week": lunch }),
        )),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{refused}");
    assert_eq!(refused["details"]["field"], json!("schedule.week[4]"));

    app.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn spec_rh_2_3_overlapping_contracts_are_refused_and_a_successor_is_not() {
    let app = harness::start().await;
    let member = colleague(&app).await;
    let contracts = member_path(&app, member, "contracts");
    let schedule = json!({ "kind": "weekly", "week": week() });
    let send = |start: &str, end: Option<&str>| {
        let body = contract(start, end, schedule.clone());
        let (app, contracts) = (&app, &contracts);
        async move {
            call(
                app,
                &app.token,
                reqwest::Method::POST,
                contracts,
                Some(body),
            )
            .await
        }
    };

    let (status, first) = send("2026-01-01", Some("2026-06-30")).await;
    assert_eq!(status, StatusCode::CREATED, "{first}");
    let (status, overlap) = send("2026-06-30", None).await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{overlap}");
    assert_eq!(overlap["code"], json!("E_MEMBER_CONTRACT_OVERLAP"));
    let (status, second) = send("2026-07-01", None).await;
    assert_eq!(status, StatusCode::CREATED, "{second}");

    let one = format!(
        "{contracts}/{}",
        first["data"]["id"].as_str().expect("an id")
    );
    let stretched = contract("2026-01-01", Some("2026-07-15"), schedule.clone());
    let (status, body) = call(
        &app,
        &app.token,
        reqwest::Method::PUT,
        &one,
        Some(stretched),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{body}");
    let shortened = contract("2026-01-01", Some("2026-06-15"), schedule);
    let (status, body) = call(
        &app,
        &app.token,
        reqwest::Method::PUT,
        &one,
        Some(shortened),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["data"]["end_date"], json!("2026-06-15"));

    let (status, _) = call(&app, &app.token, reqwest::Method::DELETE, &one, None).await;
    assert_eq!(status, StatusCode::NO_CONTENT);
    let (status, _) = call(&app, &app.token, reqwest::Method::DELETE, &one, None).await;
    assert_eq!(status, StatusCode::NOT_FOUND);

    app.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn spec_rh_2_4_a_trial_end_outside_the_contract_is_refused() {
    let app = harness::start().await;
    let member = colleague(&app).await;
    let contracts = member_path(&app, member, "contracts");
    let mut body = contract(
        "2026-03-01",
        Some("2026-09-30"),
        json!({ "kind": "weekly", "week": week() }),
    );
    body["type"] = json!("CDD");
    body["trial_end_date"] = json!("2026-10-01");
    let (status, refused) = call(
        &app,
        &app.token,
        reqwest::Method::POST,
        &contracts,
        Some(body.clone()),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{refused}");
    assert_eq!(refused["details"]["field"], json!("trial_end_date"));

    body["trial_end_date"] = json!("2026-03-31");
    let (status, created) = call(
        &app,
        &app.token,
        reqwest::Method::POST,
        &contracts,
        Some(body),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "{created}");
    assert_eq!(created["data"]["trial_end_date"], json!("2026-03-31"));

    app.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn spec_rh_2_5_a_document_is_stored_with_its_file_key_and_deleted_as_a_row_only() {
    let app = harness::start().await;
    let member = colleague(&app).await;
    let documents = member_path(&app, member, "documents");
    let body = json!({ "name": "Visite medicale", "kind": "MEDICAL", "file_key": "uploads/member-documents/abc" });
    let (status, created) = call(
        &app,
        &app.token,
        reqwest::Method::POST,
        &documents,
        Some(body),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "{created}");
    assert_eq!(
        created["data"]["file_key"],
        json!("uploads/member-documents/abc")
    );
    assert_eq!(created["data"]["kind"], json!("MEDICAL"));

    let (status, refused) = call(
        &app,
        &app.token,
        reqwest::Method::POST,
        &documents,
        Some(json!({ "name": " ", "kind": "OTHER", "file_key": "k" })),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{refused}");
    assert_eq!(refused["details"]["field"], json!("name"));

    let list = format!("/organizations/{}/member-documents", app.organization_id);
    let (status, listed) = call(&app, &app.token, reqwest::Method::GET, &list, None).await;
    assert_eq!(status, StatusCode::OK, "{listed}");
    assert_eq!(listed["data"].as_array().expect("a list").len(), 1);

    let one = format!(
        "{documents}/{}",
        created["data"]["id"].as_str().expect("an id")
    );
    let (status, _) = call(&app, &app.token, reqwest::Method::DELETE, &one, None).await;
    assert_eq!(status, StatusCode::NO_CONTENT);
    let (_, listed) = call(&app, &app.token, reqwest::Method::GET, &list, None).await;
    assert_eq!(listed["data"].as_array().expect("a list").len(), 0);
    let (status, _) = call(&app, &app.token, reqwest::Method::DELETE, &one, None).await;
    assert_eq!(status, StatusCode::NOT_FOUND);

    app.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn contracts_and_documents_need_the_member_permission_and_stop_at_the_organization() {
    let app = harness::start().await;
    let member = colleague(&app).await;
    let foreign = foreign_member(&app).await;
    let contracts = member_path(&app, member, "contracts");
    let body = contract(
        "2026-01-01",
        None,
        json!({ "kind": "weekly", "week": week() }),
    );
    let doc = json!({ "name": "CNI", "kind": "IDENTITY", "file_key": "k/1" });
    let organization = format!("/organizations/{}", app.organization_id);

    for (method, path, payload) in [
        (reqwest::Method::GET, contracts.clone(), None),
        (reqwest::Method::POST, contracts.clone(), Some(body.clone())),
        (
            reqwest::Method::GET,
            format!("{organization}/member-contracts"),
            None,
        ),
        (
            reqwest::Method::GET,
            format!("{organization}/member-documents"),
            None,
        ),
        (
            reqwest::Method::POST,
            member_path(&app, member, "documents"),
            Some(doc.clone()),
        ),
    ] {
        let (status, _) = call(&app, &app.restricted_token, method.clone(), &path, payload).await;
        assert_eq!(status, StatusCode::FORBIDDEN, "{method} {path}");
    }

    for (method, tail, payload) in [
        (reqwest::Method::GET, "contracts", None),
        (reqwest::Method::POST, "contracts", Some(body)),
        (reqwest::Method::POST, "documents", Some(doc)),
    ] {
        let (status, answer) = call(
            &app,
            &app.token,
            method.clone(),
            &member_path(&app, foreign, tail),
            payload,
        )
        .await;
        assert_eq!(status, StatusCode::NOT_FOUND, "{method} {tail}: {answer}");
    }

    app.cleanup().await;
}
