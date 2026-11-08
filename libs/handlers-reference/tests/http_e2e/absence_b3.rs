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

fn types_path(app: &App) -> String {
    format!("/organizations/{}/absence-types", app.organization_id)
}

fn absences_path(app: &App) -> String {
    format!("/organizations/{}/absences", app.organization_id)
}

fn type_body(name: &str, needs_approval: bool, needs_proof: bool) -> Value {
    json!({
        "name": name,
        "hue": 120,
        "count_mode": "WORKING",
        "account_id": null,
        "paid": true,
        "needs_approval": needs_approval,
        "needs_proof": needs_proof,
        "kind": "LEAVE",
    })
}

async fn new_type(app: &App, name: &str, needs_approval: bool, needs_proof: bool) -> String {
    let (status, created) = call(
        app,
        &app.token,
        reqwest::Method::POST,
        &types_path(app),
        Some(type_body(name, needs_approval, needs_proof)),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "{created}");
    created["data"]["id"].as_str().expect("an id").to_owned()
}

fn request(member: Uuid, type_id: &str) -> Value {
    json!({
        "member_id": member,
        "type_id": type_id,
        "start_date": "2026-07-06",
        "start_part": "AFTERNOON",
        "end_date": "2026-07-08",
        "end_part": "MORNING",
        "note": "vacances",
    })
}

async fn post(app: &App, token: &str, path: &str, body: Option<Value>) -> (StatusCode, Value) {
    call(app, token, reqwest::Method::POST, path, body).await
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn spec_rh_3_1_the_first_read_creates_the_default_types_once() {
    let app = harness::start().await;

    for _ in 0..2 {
        let (status, listed) = call(
            &app,
            &app.restricted_token,
            reqwest::Method::GET,
            &types_path(&app),
            None,
        )
        .await;
        assert_eq!(status, StatusCode::OK, "{listed}");
        let rows = listed["data"].as_array().expect("a list");
        assert_eq!(rows.len(), 7);
        assert_eq!(rows[0]["name"], json!("Congés payés"));
        assert_eq!(rows[0]["needs_approval"], json!(true));
        assert_eq!(rows[2]["kind"], json!("SICK"));
        assert_eq!(rows[2]["count_mode"], json!("CALENDAR"));
    }

    app.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn types_are_written_with_reference_manage_and_validated() {
    let app = harness::start().await;

    let (status, _) = post(
        &app,
        &app.restricted_token,
        &types_path(&app),
        Some(type_body("Interdit", true, false)),
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN);

    let (status, refused) = post(
        &app,
        &app.token,
        &types_path(&app),
        Some(
            json!({ "name": " ", "hue": 10, "count_mode": "WORKING", "paid": true,
                     "needs_approval": true, "needs_proof": false, "kind": "LEAVE" }),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{refused}");
    assert_eq!(refused["details"]["field"], json!("name"));

    let (status, refused) = post(
        &app,
        &app.token,
        &types_path(&app),
        Some(
            json!({ "name": "Teinte", "hue": 361, "count_mode": "WORKING", "paid": true,
                     "needs_approval": true, "needs_proof": false, "kind": "LEAVE" }),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{refused}");
    assert_eq!(refused["details"]["field"], json!("hue"));

    let id = new_type(&app, "Deuil", true, false).await;
    let (status, updated) = call(
        &app,
        &app.token,
        reqwest::Method::PUT,
        &format!("{}/{id}", types_path(&app)),
        Some(type_body("Deuil proche", false, true)),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{updated}");
    assert_eq!(updated["data"]["name"], json!("Deuil proche"));
    assert_eq!(updated["data"]["needs_proof"], json!(true));

    app.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn spec_rh_3_2_and_3_3_a_request_is_pending_then_decided_once() {
    let app = harness::start().await;
    let member = colleague(&app).await;
    let needing = new_type(&app, "Sur accord", true, false).await;
    let free = new_type(&app, "Sans accord", false, false).await;

    let (status, created) = post(
        &app,
        &app.token,
        &absences_path(&app),
        Some(request(member, &needing)),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "{created}");
    let absence = &created["data"];
    assert_eq!(absence["status"], json!("PENDING"));
    assert_eq!(absence["type_id"], json!(needing));
    assert_eq!(absence["start_part"], json!("AFTERNOON"));
    assert_eq!(absence["end_date"], json!("2026-07-08"));
    assert_eq!(absence["starts_at"], json!("2026-07-06T10:00:00Z"));
    assert_eq!(absence["ends_at"], json!("2026-07-08T10:00:00Z"));
    assert!(absence["created_by_member_id"].is_string());
    assert!(absence["decided_by_member_id"].is_null());
    let id = absence["id"].as_str().expect("an id").to_owned();

    let (status, immediate) = post(
        &app,
        &app.token,
        &absences_path(&app),
        Some(request(member, &free)),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "{immediate}");
    assert_eq!(immediate["data"]["status"], json!("APPROVED"));
    assert!(immediate["data"]["decided_by_member_id"].is_null());

    let (status, forbidden) = post(
        &app,
        &app.restricted_token,
        &format!("{}/{id}/approve", absences_path(&app)),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN, "{forbidden}");

    let (status, approved) = post(
        &app,
        &app.token,
        &format!("{}/{id}/approve", absences_path(&app)),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{approved}");
    assert_eq!(approved["data"]["status"], json!("APPROVED"));
    assert!(approved["data"]["decided_by_member_id"].is_string());
    assert!(approved["data"]["decided_at"].is_string());

    let (status, again) = post(
        &app,
        &app.token,
        &format!("{}/{id}/approve", absences_path(&app)),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{again}");
    assert_eq!(again["details"]["field"], json!("status"));

    let (status, listed) = call(
        &app,
        &app.token,
        reqwest::Method::GET,
        &absences_path(&app),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{listed}");
    let rows = listed["data"].as_array().expect("a list");
    assert!(rows.iter().all(|row| row["status"].is_string()));

    app.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn spec_rh_3_4_rejecting_needs_a_comment() {
    let app = harness::start().await;
    let member = colleague(&app).await;
    let needing = new_type(&app, "Sur accord", true, false).await;
    let (_, created) = post(
        &app,
        &app.token,
        &absences_path(&app),
        Some(request(member, &needing)),
    )
    .await;
    let id = created["data"]["id"].as_str().expect("an id").to_owned();
    let reject = format!("{}/{id}/reject", absences_path(&app));

    let (status, refused) = post(&app, &app.token, &reject, Some(json!({ "comment": "  " }))).await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{refused}");
    assert_eq!(refused["details"]["field"], json!("comment"));

    let (status, rejected) = post(
        &app,
        &app.token,
        &reject,
        Some(json!({ "comment": "période chargée" })),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{rejected}");
    assert_eq!(rejected["data"]["status"], json!("REJECTED"));
    assert_eq!(
        rejected["data"]["decision_comment"],
        json!("période chargée")
    );

    app.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn spec_rh_3_5_a_type_needing_proof_refuses_approval_without_one() {
    let app = harness::start().await;
    let member = colleague(&app).await;
    let proofed = new_type(&app, "Sur justificatif", true, true).await;
    let (_, created) = post(
        &app,
        &app.token,
        &absences_path(&app),
        Some(request(member, &proofed)),
    )
    .await;
    let id = created["data"]["id"].as_str().expect("an id").to_owned();
    let approve = format!("{}/{id}/approve", absences_path(&app));

    let (status, refused) = post(&app, &app.token, &approve, None).await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{refused}");
    assert_eq!(refused["details"]["field"], json!("proof_file_key"));

    let (status, with_proof) = post(
        &app,
        &app.token,
        &absences_path(&app),
        Some(json!({
            "member_id": member,
            "type_id": proofed,
            "start_date": "2026-09-01",
            "end_date": "2026-09-01",
            "proof_file_key": "absences/certificat.pdf",
        })),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "{with_proof}");
    let proofed_id = with_proof["data"]["id"].as_str().expect("an id");
    let (status, approved) = post(
        &app,
        &app.token,
        &format!("{}/{proofed_id}/approve", absences_path(&app)),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{approved}");

    app.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn spec_rh_3_6_an_archived_type_is_refused_and_stays_on_past_absences() {
    let app = harness::start().await;
    let member = colleague(&app).await;
    let type_id = new_type(&app, "Ancien motif", true, false).await;
    let (_, past) = post(
        &app,
        &app.token,
        &absences_path(&app),
        Some(request(member, &type_id)),
    )
    .await;
    let past_id = past["data"]["id"].as_str().expect("an id").to_owned();

    let (status, archived) = post(
        &app,
        &app.token,
        &format!("{}/{type_id}/archive", types_path(&app)),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{archived}");
    assert!(archived["data"]["archived_at"].is_string());

    let (status, refused) = post(
        &app,
        &app.token,
        &absences_path(&app),
        Some(request(member, &type_id)),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{refused}");
    assert_eq!(refused["details"]["field"], json!("type_id"));

    let (_, listed) = call(
        &app,
        &app.token,
        reqwest::Method::GET,
        &absences_path(&app),
        None,
    )
    .await;
    let kept = listed["data"]
        .as_array()
        .expect("a list")
        .iter()
        .find(|row| row["id"] == json!(past_id))
        .expect("the past absence is still listed");
    assert_eq!(kept["type_id"], json!(type_id));

    let (status, restored) = post(
        &app,
        &app.token,
        &format!("{}/{type_id}/restore", types_path(&app)),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{restored}");
    assert!(restored["data"]["archived_at"].is_null());

    app.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn the_original_request_shape_still_works_and_cancel_follows_its_rule() {
    let app = harness::start().await;
    let member = colleague(&app).await;

    let (status, created) = post(
        &app,
        &app.token,
        &absences_path(&app),
        Some(json!({
            "member_id": member,
            "kind": "SICK",
            "starts_at": "2026-02-02T08:00:00Z",
            "ends_at": "2026-02-02T16:00:00Z",
            "all_day": false,
            "note": "grippe",
        })),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "{created}");
    assert_eq!(created["data"]["status"], json!("APPROVED"));
    assert_eq!(created["data"]["kind"], json!("SICK"));
    assert_eq!(created["data"]["starts_at"], json!("2026-02-02T08:00:00Z"));
    let id = created["data"]["id"].as_str().expect("an id").to_owned();
    let cancel = format!("{}/{id}/cancel", absences_path(&app));

    let (status, refused) = post(&app, &app.restricted_token, &cancel, None).await;
    assert_eq!(status, StatusCode::FORBIDDEN, "{refused}");

    let (status, cancelled) = post(&app, &app.token, &cancel, None).await;
    assert_eq!(status, StatusCode::OK, "{cancelled}");
    assert_eq!(cancelled["data"]["status"], json!("CANCELLED"));

    let (status, neither) = post(
        &app,
        &app.token,
        &absences_path(&app),
        Some(
            json!({ "member_id": member, "starts_at": "2026-02-02T08:00:00Z",
                     "ends_at": "2026-02-02T16:00:00Z" }),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{neither}");
    assert_eq!(neither["details"]["field"], json!("type_id"));

    app.cleanup().await;
}
