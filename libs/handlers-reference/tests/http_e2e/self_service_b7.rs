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

async fn seat(app: &App, last_name: &str) -> Uuid {
    sqlx::query_scalar(
        "SELECT id FROM organization_members WHERE organization_id = $1 AND last_name = $2",
    )
    .bind(app.organization_id)
    .bind(last_name)
    .fetch_one(&app.pool)
    .await
    .expect("a seeded seat")
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

fn org(app: &App, suffix: &str) -> String {
    format!("/organizations/{}/{suffix}", app.organization_id)
}

async fn new_type(app: &App, name: &str, needs_approval: bool) -> String {
    let (status, created) = post(
        app,
        &app.token,
        &org(app, "absence-types"),
        Some(json!({
            "name": name,
            "hue": 120,
            "count_mode": "WORKING",
            "account_id": null,
            "paid": true,
            "needs_approval": needs_approval,
            "needs_proof": false,
            "kind": "LEAVE",
        })),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "{created}");
    created["data"]["id"].as_str().expect("an id").to_owned()
}

fn absence_body(member: Uuid, type_id: &str) -> Value {
    json!({
        "member_id": member,
        "type_id": type_id,
        "start_date": "2026-07-06",
        "end_date": "2026-07-07",
        "note": "vacances",
        "proof_file_key": "absences/certificat.pdf",
    })
}

fn extra_body(member: Uuid) -> Value {
    json!({
        "member_id": member,
        "date": "2026-11-14",
        "start": "08:00",
        "end": "12:00",
        "reason": "Rattrapage du chantier",
        "approve": true,
    })
}

async fn id_of(created: (StatusCode, Value)) -> String {
    assert_eq!(created.0, StatusCode::CREATED, "{}", created.1);
    created.1["data"]["id"].as_str().expect("an id").to_owned()
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn spec_rh_7_2_a_member_requests_for_themselves_and_is_refused_for_another() {
    let app = harness::start().await;
    let me = seat(&app, "No Role").await;
    let other = colleague(&app).await;
    let leave = new_type(&app, "Conges libre-service", true).await;

    let (status, created) = post(
        &app,
        &app.restricted_token,
        &org(&app, "absences"),
        Some(absence_body(me, &leave)),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "{created}");
    assert_eq!(created["data"]["member_id"], json!(me));
    assert_eq!(
        created["data"]["proof_file_key"],
        json!("absences/certificat.pdf")
    );

    let (status, refused) = post(
        &app,
        &app.restricted_token,
        &org(&app, "absences"),
        Some(absence_body(other, &leave)),
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN, "{refused}");

    let (status, window) = post(
        &app,
        &app.restricted_token,
        &org(&app, "absences"),
        Some(json!({
            "member_id": me,
            "kind": "LEAVE",
            "starts_at": "2026-07-06T08:00:00Z",
            "ends_at": "2026-07-06T10:00:00Z",
        })),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{window}");
    assert_eq!(window["details"]["field"], json!("type_id"));

    let (status, created) = post(
        &app,
        &app.restricted_token,
        &org(&app, "extra-periods"),
        Some(extra_body(me)),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "{created}");

    let (status, refused) = post(
        &app,
        &app.restricted_token,
        &org(&app, "extra-periods"),
        Some(extra_body(other)),
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN, "{refused}");

    let (status, managed) = post(
        &app,
        &app.token,
        &org(&app, "absences"),
        Some(absence_body(other, &leave)),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "{managed}");

    app.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn spec_rh_7_3_a_self_extra_is_pending_and_a_self_absence_follows_its_type() {
    let app = harness::start().await;
    let me = seat(&app, "No Role").await;
    let owner = seat(&app, "Owner").await;
    let approval = new_type(&app, "Avec validation", true).await;
    let free = new_type(&app, "Sans validation", false).await;

    let (status, extra) = post(
        &app,
        &app.restricted_token,
        &org(&app, "extra-periods"),
        Some(extra_body(me)),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "{extra}");
    assert_eq!(extra["data"]["status"], json!("PENDING"));
    assert_eq!(extra["data"]["created_by_member_id"], json!(me));

    let (status, managed) = post(
        &app,
        &app.token,
        &org(&app, "extra-periods"),
        Some(extra_body(owner)),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "{managed}");
    assert_eq!(managed["data"]["status"], json!("APPROVED"));

    for (type_id, expected) in [(&approval, "PENDING"), (&free, "APPROVED")] {
        let (status, absence) = post(
            &app,
            &app.restricted_token,
            &org(&app, "absences"),
            Some(absence_body(me, type_id)),
        )
        .await;
        assert_eq!(status, StatusCode::CREATED, "{absence}");
        assert_eq!(absence["data"]["status"], json!(expected));
    }

    app.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn spec_rh_7_4_a_member_cancels_their_own_pending_request_only() {
    let app = harness::start().await;
    let me = seat(&app, "No Role").await;
    let leave = new_type(&app, "Conges a annuler", true).await;
    let absences = org(&app, "absences");
    let extras = org(&app, "extra-periods");

    let own_absence = id_of(
        post(
            &app,
            &app.restricted_token,
            &absences,
            Some(absence_body(me, &leave)),
        )
        .await,
    )
    .await;
    let (status, cancelled) = post(
        &app,
        &app.restricted_token,
        &format!("{absences}/{own_absence}/cancel"),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{cancelled}");
    assert_eq!(cancelled["data"]["status"], json!("CANCELLED"));

    let decided = id_of(
        post(
            &app,
            &app.restricted_token,
            &absences,
            Some(absence_body(me, &leave)),
        )
        .await,
    )
    .await;
    let (status, approved) = post(
        &app,
        &app.token,
        &format!("{absences}/{decided}/approve"),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{approved}");
    let (status, refused) = post(
        &app,
        &app.restricted_token,
        &format!("{absences}/{decided}/cancel"),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{refused}");
    assert_eq!(refused["details"]["field"], json!("status"));

    let by_manager =
        id_of(post(&app, &app.token, &absences, Some(absence_body(me, &leave))).await).await;
    let (status, refused) = post(
        &app,
        &app.restricted_token,
        &format!("{absences}/{by_manager}/cancel"),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN, "{refused}");

    let own_extra =
        id_of(post(&app, &app.restricted_token, &extras, Some(extra_body(me))).await).await;
    let (status, cancelled) = post(
        &app,
        &app.restricted_token,
        &format!("{extras}/{own_extra}/cancel"),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{cancelled}");
    assert_eq!(cancelled["data"]["status"], json!("CANCELLED"));

    let decided =
        id_of(post(&app, &app.restricted_token, &extras, Some(extra_body(me))).await).await;
    let (status, approved) = post(
        &app,
        &app.token,
        &format!("{extras}/{decided}/approve"),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{approved}");
    let (status, refused) = post(
        &app,
        &app.restricted_token,
        &format!("{extras}/{decided}/cancel"),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{refused}");

    let by_manager = id_of(post(&app, &app.token, &extras, Some(extra_body(me))).await).await;
    let (status, refused) = post(
        &app,
        &app.restricted_token,
        &format!("{extras}/{by_manager}/cancel"),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN, "{refused}");

    app.cleanup().await;
}

fn member_ids(listed: &Value) -> Vec<String> {
    listed["data"]
        .as_array()
        .expect("a list")
        .iter()
        .map(|row| row["member_id"].as_str().expect("a member").to_owned())
        .collect()
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn spec_rh_7_5_a_member_lists_only_their_own_requests_and_every_account() {
    let app = harness::start().await;
    let me = seat(&app, "No Role").await;
    let owner = seat(&app, "Owner").await;
    let other = colleague(&app).await;
    let leave = new_type(&app, "Conges lecture", true).await;

    for member in [me, owner, other] {
        let (status, created) = post(
            &app,
            &app.token,
            &org(&app, "absences"),
            Some(absence_body(member, &leave)),
        )
        .await;
        assert_eq!(status, StatusCode::CREATED, "{created}");
        let (status, created) = post(
            &app,
            &app.token,
            &org(&app, "extra-periods"),
            Some(extra_body(member)),
        )
        .await;
        assert_eq!(status, StatusCode::CREATED, "{created}");
    }
    let (_, accounts) = get(&app, &app.token, &org(&app, "leave-accounts")).await;
    let account = accounts["data"][0]["id"].as_str().expect("an account");
    for member in [me, owner, other] {
        let (status, created) = post(
            &app,
            &app.token,
            &org(&app, "leave-adjustments"),
            Some(json!({
                "member_id": member,
                "account_id": account,
                "date": "2026-07-01",
                "days": 1.0,
                "kind": "ADJUSTMENT",
                "reason": "geste",
            })),
        )
        .await;
        assert_eq!(status, StatusCode::CREATED, "{created}");
    }

    for resource in ["absences", "extra-periods", "leave-adjustments"] {
        let path = org(&app, resource);
        let (status, own) = get(&app, &app.restricted_token, &path).await;
        assert_eq!(status, StatusCode::OK, "{resource}: {own}");
        assert_eq!(member_ids(&own), vec![me.to_string()], "{resource}");

        let (status, same) = get(
            &app,
            &app.restricted_token,
            &format!("{path}?member_id={me}"),
        )
        .await;
        assert_eq!(status, StatusCode::OK, "{resource}: {same}");
        assert_eq!(member_ids(&same), vec![me.to_string()]);

        let (status, refused) = get(
            &app,
            &app.restricted_token,
            &format!("{path}?member_id={other}"),
        )
        .await;
        assert_eq!(status, StatusCode::FORBIDDEN, "{resource}: {refused}");

        let (status, all) = get(&app, &app.token, &path).await;
        assert_eq!(status, StatusCode::OK, "{resource}: {all}");
        assert_eq!(member_ids(&all).len(), 3, "{resource}");
    }

    let (status, listed) = get(&app, &app.restricted_token, &org(&app, "leave-accounts")).await;
    assert_eq!(status, StatusCode::OK, "{listed}");
    assert_eq!(listed["data"].as_array().expect("a list").len(), 2);
    let (status, listed) = get(&app, &app.restricted_token, &org(&app, "absence-types")).await;
    assert_eq!(status, StatusCode::OK, "{listed}");

    app.cleanup().await;
}
