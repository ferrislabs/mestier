use reqwest::{Client, Response, StatusCode};
use serde_json::{Value, json};

use crate::harness::{self, App};

async fn json_body(response: Response) -> Value {
    response.json().await.unwrap()
}

fn settings_url(app: &App) -> String {
    app.support_url("/settings")
}

fn other_settings_url(app: &App) -> String {
    format!(
        "{}/api/v1/organizations/{}/support/settings",
        app.base_url, app.other_organization_id
    )
}

async fn put(app: &App, token: &str, display_fields: Value) -> Response {
    Client::new()
        .put(settings_url(app))
        .bearer_auth(token)
        .json(&json!({ "display_fields": display_fields }))
        .send()
        .await
        .unwrap()
}

async fn get(app: &App, token: &str) -> Response {
    Client::new()
        .get(settings_url(app))
        .bearer_auth(token)
        .send()
        .await
        .unwrap()
}

async fn rows(app: &App) -> i64 {
    sqlx::query_scalar("SELECT COUNT(*) FROM support_settings WHERE org_id = $1")
        .bind(app.organization_id)
        .fetch_one(&app.pool)
        .await
        .unwrap()
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn anonymous_and_outsider_are_refused_on_get_and_put() {
    let app = harness::start().await;
    let client = Client::new();

    let anonymous_get = client.get(settings_url(&app)).send().await.unwrap();
    let anonymous_put = client
        .put(settings_url(&app))
        .json(&json!({ "display_fields": ["team"] }))
        .send()
        .await
        .unwrap();
    let outsider_get = get(&app, &app.outsider.token).await;
    let outsider_put = put(&app, &app.outsider.token, json!(["team"])).await;

    assert_eq!(anonymous_get.status(), StatusCode::UNAUTHORIZED);
    assert_eq!(anonymous_put.status(), StatusCode::UNAUTHORIZED);
    assert_eq!(outsider_get.status(), StatusCode::FORBIDDEN);
    assert_eq!(outsider_put.status(), StatusCode::FORBIDDEN);
    assert_eq!(rows(&app).await, 0);
    app.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn get_without_a_row_answers_an_empty_list_and_writes_nothing() {
    let app = harness::start().await;

    let response = get(&app, &app.viewer.token).await;

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        json_body(response).await,
        json!({ "data": { "display_fields": [] } })
    );
    assert_eq!(rows(&app).await, 0);
    app.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn put_keeps_the_order_and_get_reads_it_back() {
    let app = harness::start().await;

    let replaced = put(
        &app,
        &app.admin.token,
        json!(["plan", "team", "deployment_id"]),
    )
    .await;

    assert_eq!(replaced.status(), StatusCode::OK);
    assert_eq!(
        json_body(replaced).await,
        json!({ "data": { "display_fields": ["plan", "team", "deployment_id"] } })
    );
    let read = get(&app, &app.viewer.token).await;
    assert_eq!(read.status(), StatusCode::OK);
    assert_eq!(
        json_body(read).await,
        json!({ "data": { "display_fields": ["plan", "team", "deployment_id"] } })
    );
    assert_eq!(rows(&app).await, 1);
    app.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn a_second_put_overwrites_the_first() {
    let app = harness::start().await;

    put(&app, &app.admin.token, json!(["plan", "team"])).await;
    let second = put(&app, &app.admin.token, json!(["seats"])).await;
    let cleared = put(&app, &app.admin.token, json!([])).await;

    assert_eq!(second.status(), StatusCode::OK);
    assert_eq!(
        json_body(second).await,
        json!({ "data": { "display_fields": ["seats"] } })
    );
    assert_eq!(cleared.status(), StatusCode::OK);
    assert_eq!(
        json_body(cleared).await,
        json!({ "data": { "display_fields": [] } })
    );
    assert_eq!(rows(&app).await, 1);
    app.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn put_refuses_a_duplicate_key_with_422() {
    let app = harness::start().await;

    let response = put(&app, &app.admin.token, json!(["team", "plan", "team"])).await;

    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
    let body = json_body(response).await;
    assert_eq!(body["code"], "E_UNPROCESSABLE_ENTITY");
    assert!(body["message"].as_str().unwrap().contains("team"));
    assert_eq!(rows(&app).await, 0);
    app.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn put_accepts_eight_keys_and_refuses_a_ninth_with_422() {
    let app = harness::start().await;
    let eight: Vec<String> = (0..8).map(|index| format!("key{index}")).collect();
    let nine: Vec<String> = (0..9).map(|index| format!("key{index}")).collect();

    let accepted = put(&app, &app.admin.token, json!(eight)).await;
    let refused = put(&app, &app.admin.token, json!(nine)).await;

    assert_eq!(accepted.status(), StatusCode::OK);
    assert_eq!(refused.status(), StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(
        json_body(get(&app, &app.admin.token).await).await["data"]["display_fields"],
        json!(eight)
    );
    app.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn put_refuses_an_empty_key_with_422() {
    let app = harness::start().await;

    let response = put(&app, &app.admin.token, json!(["team", ""])).await;

    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(rows(&app).await, 0);
    app.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn put_accepts_a_64_character_key_and_refuses_65_with_422() {
    let app = harness::start().await;
    let accepted_key = "k".repeat(64);
    let accented_key = "é".repeat(64);

    let accepted = put(&app, &app.admin.token, json!([accepted_key])).await;
    let accented = put(&app, &app.admin.token, json!([accented_key])).await;
    let refused = put(&app, &app.admin.token, json!(["k".repeat(65)])).await;
    let refused_accented = put(&app, &app.admin.token, json!(["é".repeat(65)])).await;

    assert_eq!(accepted.status(), StatusCode::OK);
    assert_eq!(accented.status(), StatusCode::OK);
    assert_eq!(
        json_body(accented).await["data"]["display_fields"],
        json!([accented_key])
    );
    assert_eq!(refused.status(), StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(refused_accented.status(), StatusCode::UNPROCESSABLE_ENTITY);
    app.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn a_viewer_reads_with_200_but_is_refused_on_put_with_403() {
    let app = harness::start().await;
    put(&app, &app.admin.token, json!(["plan"])).await;

    let read = get(&app, &app.viewer.token).await;
    let write = put(&app, &app.viewer.token, json!(["team"])).await;

    assert_eq!(read.status(), StatusCode::OK);
    assert_eq!(
        json_body(read).await,
        json!({ "data": { "display_fields": ["plan"] } })
    );
    assert_eq!(write.status(), StatusCode::FORBIDDEN);
    assert_eq!(
        json_body(get(&app, &app.admin.token).await).await["data"]["display_fields"],
        json!(["plan"])
    );
    app.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn settings_of_another_organization_stay_out_of_reach() {
    let app = harness::start().await;
    let client = Client::new();

    let read = client
        .get(other_settings_url(&app))
        .bearer_auth(&app.admin.token)
        .send()
        .await
        .unwrap();
    let write = client
        .put(other_settings_url(&app))
        .bearer_auth(&app.admin.token)
        .json(&json!({ "display_fields": ["team"] }))
        .send()
        .await
        .unwrap();

    assert_eq!(read.status(), StatusCode::FORBIDDEN);
    assert_eq!(write.status(), StatusCode::FORBIDDEN);
    let other_rows: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM support_settings WHERE org_id = $1")
            .bind(app.other_organization_id)
            .fetch_one(&app.pool)
            .await
            .unwrap();
    assert_eq!(other_rows, 0);
    app.cleanup().await;
}
