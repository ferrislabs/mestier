use std::time::Duration;

use chrono::{DateTime, Utc};
use mestier_core::PresentedApiKey;
use reqwest::{Client, Response, StatusCode};
use serde_json::{Value, json};
use uuid::Uuid;

use crate::harness::{self, App};

const UNAUTHORIZED_BODY: &str =
    r#"{"code":"E_UNAUTHORIZED","message":"Unauthorized","status":401}"#;

fn is_lowercase_hex(value: &str, length: usize) -> bool {
    value.len() == length
        && value
            .bytes()
            .all(|byte| matches!(byte, b'0'..=b'9' | b'a'..=b'f'))
}

async fn json_body(response: Response) -> Value {
    response.json().await.unwrap()
}

async fn create_key_in(
    app: &App,
    client: &Client,
    token: &str,
    organization_id: Uuid,
    name: &str,
) -> Value {
    let response = client
        .post(app.api_keys_url_in(organization_id, ""))
        .bearer_auth(token)
        .json(&json!({ "name": name }))
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);
    json_body(response).await["data"].clone()
}

async fn create_key(app: &App, client: &Client, name: &str) -> Value {
    create_key_in(
        app,
        client,
        &app.organization_admin.token,
        app.organization_id,
        name,
    )
    .await
}

fn key_id(created: &Value) -> Uuid {
    created["id"].as_str().unwrap().parse().unwrap()
}

fn plaintext(created: &Value) -> &str {
    created["key"].as_str().unwrap()
}

fn secret(created: &Value) -> &str {
    plaintext(created).split_once('.').unwrap().1
}

async fn revoke(app: &App, client: &Client, id: Uuid) -> Response {
    client
        .delete(app.api_keys_url(&format!("/{id}")))
        .bearer_auth(&app.organization_admin.token)
        .send()
        .await
        .unwrap()
}

async fn list(app: &App, client: &Client) -> (String, Value) {
    let response = client
        .get(app.api_keys_url(""))
        .bearer_auth(&app.organization_admin.token)
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let text = response.text().await.unwrap();
    let body = serde_json::from_str(&text).unwrap();

    (text, body)
}

async fn probe(app: &App, client: &Client, authorization: Option<&str>) -> (StatusCode, String) {
    let mut request = client.get(app.probe_url());
    if let Some(authorization) = authorization {
        request = request.header("authorization", authorization);
    }
    let response = request.send().await.unwrap();
    let status = response.status();

    (status, response.text().await.unwrap())
}

async fn last_used_at(app: &App, id: Uuid) -> Option<DateTime<Utc>> {
    sqlx::query_scalar("SELECT last_used_at FROM support_api_keys WHERE id = $1")
        .bind(id)
        .fetch_one(&app.pool)
        .await
        .unwrap()
}

async fn revoked_at(app: &App, id: Uuid) -> Option<DateTime<Utc>> {
    sqlx::query_scalar("SELECT revoked_at FROM support_api_keys WHERE id = $1")
        .bind(id)
        .fetch_one(&app.pool)
        .await
        .unwrap()
}

async fn key_rows(app: &App) -> i64 {
    sqlx::query_scalar("SELECT COUNT(*) FROM support_api_keys WHERE org_id = ANY($1)")
        .bind(vec![app.organization_id, app.other_organization_id])
        .fetch_one(&app.pool)
        .await
        .unwrap()
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn an_organization_administrator_creates_a_key_and_receives_the_plaintext() {
    let app = harness::start().await;
    let client = Client::new();

    let response = client
        .post(app.api_keys_url(""))
        .bearer_auth(&app.organization_admin.token)
        .json(&json!({ "name": "  Formulaire de contact  " }))
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);
    let body = json_body(response).await;
    let data = &body["data"];
    let mut fields: Vec<&str> = data
        .as_object()
        .unwrap()
        .keys()
        .map(String::as_str)
        .collect();
    fields.sort_unstable();
    assert_eq!(fields, ["created_at", "id", "key", "name", "prefix"]);
    assert_eq!(data["name"], "Formulaire de contact");
    assert!(data["created_at"].is_string());
    let (prefix, secret) = plaintext(data).split_once('.').unwrap();
    assert_eq!(prefix, data["prefix"].as_str().unwrap());
    assert!(is_lowercase_hex(prefix, 8), "{prefix}");
    assert!(is_lowercase_hex(secret, 64));
    let stored_hash: Vec<u8> =
        sqlx::query_scalar("SELECT secret_hash FROM support_api_keys WHERE id = $1")
            .bind(key_id(data))
            .fetch_one(&app.pool)
            .await
            .unwrap();
    assert_eq!(
        stored_hash,
        PresentedApiKey::parse(plaintext(data))
            .unwrap()
            .secret_hash()
    );
    app.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn a_blank_or_overlong_name_is_refused_with_a_409_and_nothing_is_written() {
    let app = harness::start().await;
    let client = Client::new();

    for name in ["", "   ", &"x".repeat(101)] {
        let response = client
            .post(app.api_keys_url(""))
            .bearer_auth(&app.organization_admin.token)
            .json(&json!({ "name": name }))
            .send()
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::CONFLICT, "{name:?}");
    }

    assert_eq!(key_rows(&app).await, 0);
    app.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn the_plaintext_appears_in_no_response_but_the_creation() {
    let app = harness::start().await;
    let client = Client::new();
    let created = create_key(&app, &client, "Formulaire").await;
    let id = key_id(&created);

    let (listing, body) = list(&app, &client).await;
    let revocation = revoke(&app, &client, id).await;
    let revocation_status = revocation.status();
    let revocation_text = revocation.text().await.unwrap();
    let (listing_after_revoke, _) = list(&app, &client).await;

    assert_eq!(revocation_status, StatusCode::NO_CONTENT);
    for text in [&listing, &revocation_text, &listing_after_revoke] {
        assert!(!text.contains(secret(&created)), "{text}");
        assert!(!text.contains(plaintext(&created)), "{text}");
    }
    let items = body["data"].as_array().unwrap();
    assert_eq!(items.len(), 1);
    let mut fields: Vec<&str> = items[0]
        .as_object()
        .unwrap()
        .keys()
        .map(String::as_str)
        .collect();
    fields.sort_unstable();
    assert_eq!(
        fields,
        [
            "created_at",
            "id",
            "last_used_at",
            "name",
            "prefix",
            "revoked_at"
        ]
    );
    assert_eq!(items[0]["prefix"], created["prefix"]);
    assert_eq!(items[0]["last_used_at"], Value::Null);
    assert_eq!(items[0]["revoked_at"], Value::Null);
    app.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn the_list_is_newest_first_and_keeps_revoked_keys() {
    let app = harness::start().await;
    let client = Client::new();
    let first = create_key(&app, &client, "Premier").await;
    let second = create_key(&app, &client, "Second").await;
    revoke(&app, &client, key_id(&first)).await;

    let (_, body) = list(&app, &client).await;

    let items = body["data"].as_array().unwrap();
    assert_eq!(items.len(), 2);
    assert_eq!(items[0]["id"], second["id"]);
    assert_eq!(items[0]["revoked_at"], Value::Null);
    assert_eq!(items[1]["id"], first["id"]);
    assert!(items[1]["revoked_at"].is_string());
    app.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn callers_without_manage_org_are_refused_on_all_three_routes() {
    let app = harness::start().await;
    let client = Client::new();
    let existing = create_key(&app, &client, "Existante").await;
    let existing_url = app.api_keys_url(&format!("/{}", key_id(&existing)));
    let collection_url = app.api_keys_url("");

    let refused = [
        ("viewer", &app.viewer.token),
        ("outsider", &app.outsider.token),
        ("support administrator", &app.admin.token),
        (
            "administrator of another organization",
            &app.other_organization_admin.token,
        ),
    ];
    for (who, token) in refused {
        let created = client
            .post(&collection_url)
            .bearer_auth(token)
            .json(&json!({ "name": "Intrus" }))
            .send()
            .await
            .unwrap();
        let listed = client
            .get(&collection_url)
            .bearer_auth(token)
            .send()
            .await
            .unwrap();
        let revoked = client
            .delete(&existing_url)
            .bearer_auth(token)
            .send()
            .await
            .unwrap();

        assert_eq!(created.status(), StatusCode::FORBIDDEN, "{who} create");
        assert_eq!(listed.status(), StatusCode::FORBIDDEN, "{who} list");
        assert_eq!(revoked.status(), StatusCode::FORBIDDEN, "{who} revoke");
    }

    let created = client
        .post(&collection_url)
        .json(&json!({ "name": "Anonyme" }))
        .send()
        .await
        .unwrap();
    let listed = client.get(&collection_url).send().await.unwrap();
    let revoked = client.delete(&existing_url).send().await.unwrap();
    assert_eq!(created.status(), StatusCode::UNAUTHORIZED);
    assert_eq!(listed.status(), StatusCode::UNAUTHORIZED);
    assert_eq!(revoked.status(), StatusCode::UNAUTHORIZED);

    assert_eq!(key_rows(&app).await, 1);
    assert_eq!(revoked_at(&app, key_id(&existing)).await, None);
    app.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn revoke_answers_204_twice_and_404_outside_the_organization() {
    let app = harness::start().await;
    let client = Client::new();
    let created = create_key(&app, &client, "Formulaire").await;
    let id = key_id(&created);
    let foreign = create_key_in(
        &app,
        &client,
        &app.other_organization_admin.token,
        app.other_organization_id,
        "Ailleurs",
    )
    .await;

    let first = revoke(&app, &client, id).await;
    let first_revocation = revoked_at(&app, id).await;
    tokio::time::sleep(Duration::from_millis(50)).await;
    let second = revoke(&app, &client, id).await;
    let unknown = revoke(&app, &client, Uuid::now_v7()).await;
    let other_organization = revoke(&app, &client, key_id(&foreign)).await;

    assert_eq!(first.status(), StatusCode::NO_CONTENT);
    assert!(first_revocation.is_some());
    assert_eq!(second.status(), StatusCode::NO_CONTENT);
    assert_eq!(revoked_at(&app, id).await, first_revocation);
    assert_eq!(unknown.status(), StatusCode::NOT_FOUND);
    assert_eq!(other_organization.status(), StatusCode::NOT_FOUND);
    assert_eq!(revoked_at(&app, key_id(&foreign)).await, None);
    app.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn a_valid_key_opens_the_probe_as_its_organization_and_last_used_at_moves() {
    let app = harness::start().await;
    let client = Client::new();
    let created = create_key(&app, &client, "Formulaire").await;
    let id = key_id(&created);
    assert_eq!(last_used_at(&app, id).await, None);

    let (status, body) = probe(
        &app,
        &client,
        Some(&format!("Bearer {}", plaintext(&created))),
    )
    .await;
    let first_use = last_used_at(&app, id).await;
    tokio::time::sleep(Duration::from_millis(50)).await;
    let (lowercase_status, lowercase_body) = probe(
        &app,
        &client,
        Some(&format!("bearer {}", plaintext(&created))),
    )
    .await;
    let second_use = last_used_at(&app, id).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body, app.organization_id.to_string());
    assert_eq!(lowercase_status, StatusCode::OK);
    assert_eq!(lowercase_body, app.organization_id.to_string());
    assert!(first_use.is_some());
    assert!(second_use > first_use);
    app.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn a_key_belongs_to_the_organization_that_created_it() {
    let app = harness::start().await;
    let client = Client::new();
    let foreign = create_key_in(
        &app,
        &client,
        &app.other_organization_admin.token,
        app.other_organization_id,
        "Ailleurs",
    )
    .await;

    let (status, body) = probe(
        &app,
        &client,
        Some(&format!("Bearer {}", plaintext(&foreign))),
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body, app.other_organization_id.to_string());
    assert_ne!(body, app.organization_id.to_string());
    app.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn a_revoked_key_gets_401_and_last_used_at_stays_put() {
    let app = harness::start().await;
    let client = Client::new();
    let created = create_key(&app, &client, "Formulaire").await;
    let id = key_id(&created);
    let authorization = format!("Bearer {}", plaintext(&created));
    let (before, _) = probe(&app, &client, Some(&authorization)).await;
    assert_eq!(before, StatusCode::OK);
    let last_use = last_used_at(&app, id).await;
    revoke(&app, &client, id).await;
    tokio::time::sleep(Duration::from_millis(50)).await;

    let (status, body) = probe(&app, &client, Some(&authorization)).await;

    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert_eq!(body, UNAUTHORIZED_BODY);
    assert_eq!(last_used_at(&app, id).await, last_use);
    app.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn a_key_of_a_soft_deleted_organization_gets_401_and_last_used_at_stays_put() {
    let app = harness::start().await;
    let client = Client::new();
    let created = create_key(&app, &client, "Formulaire").await;
    let id = key_id(&created);
    let authorization = format!("Bearer {}", plaintext(&created));
    let (before, _) = probe(&app, &client, Some(&authorization)).await;
    assert_eq!(before, StatusCode::OK);
    let last_use = last_used_at(&app, id).await;
    sqlx::query("UPDATE organizations SET deleted_at = now() WHERE id = $1")
        .bind(app.organization_id)
        .execute(&app.pool)
        .await
        .unwrap();
    tokio::time::sleep(Duration::from_millis(50)).await;

    let (status, body) = probe(&app, &client, Some(&authorization)).await;

    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert_eq!(body, UNAUTHORIZED_BODY);
    assert_eq!(last_used_at(&app, id).await, last_use);
    app.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn every_way_of_failing_to_authenticate_gets_the_same_401_body() {
    let app = harness::start().await;
    let client = Client::new();
    let created = create_key(&app, &client, "Formulaire").await;
    let revoked = create_key(&app, &client, "Révoquée").await;
    revoke(&app, &client, key_id(&revoked)).await;
    let (prefix, secret) = plaintext(&created).split_once('.').unwrap();
    let flipped = if secret.starts_with('0') { '1' } else { '0' };
    let unknown_prefix = if prefix.starts_with('0') { '1' } else { '0' };

    let attempts: Vec<(&str, Option<String>)> = vec![
        ("no header", None),
        (
            "wrong scheme",
            Some(format!("Basic {}", plaintext(&created))),
        ),
        ("no scheme", Some(plaintext(&created).to_owned())),
        (
            "double space",
            Some(format!("Bearer  {}", plaintext(&created))),
        ),
        ("empty key", Some("Bearer ".to_owned())),
        ("malformed key", Some("Bearer not-a-key".to_owned())),
        ("missing secret", Some(format!("Bearer {prefix}."))),
        ("missing dot", Some(format!("Bearer {prefix}{secret}"))),
        (
            "uppercase secret",
            Some(format!("Bearer {prefix}.{}", secret.to_uppercase())),
        ),
        (
            "wrong secret",
            Some(format!("Bearer {prefix}.{flipped}{}", &secret[1..])),
        ),
        (
            "unknown prefix",
            Some(format!("Bearer {unknown_prefix}{}.{secret}", &prefix[1..])),
        ),
        (
            "revoked key",
            Some(format!("Bearer {}", plaintext(&revoked))),
        ),
        (
            "user token",
            Some(format!("Bearer {}", app.organization_admin.token)),
        ),
        (
            "support admin token",
            Some(format!("Bearer {}", app.admin.token)),
        ),
    ];

    let mut outcomes = Vec::new();
    for (label, authorization) in &attempts {
        let (status, body) = probe(&app, &client, authorization.as_deref()).await;
        outcomes.push((*label, status, body));
    }

    for (label, status, body) in &outcomes {
        assert_eq!(*status, StatusCode::UNAUTHORIZED, "{label}");
        assert_eq!(body, UNAUTHORIZED_BODY, "{label}");
    }
    assert_eq!(last_used_at(&app, key_id(&created)).await, None);
    assert_eq!(last_used_at(&app, key_id(&revoked)).await, None);
    app.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn a_key_never_opens_a_member_route() {
    let app = harness::start().await;
    let client = Client::new();
    let created = create_key(&app, &client, "Formulaire").await;

    let tickets = client
        .get(app.tickets_url(""))
        .bearer_auth(plaintext(&created))
        .send()
        .await
        .unwrap();
    let keys = client
        .get(app.api_keys_url(""))
        .bearer_auth(plaintext(&created))
        .send()
        .await
        .unwrap();
    let revocation = client
        .delete(app.api_keys_url(&format!("/{}", key_id(&created))))
        .bearer_auth(plaintext(&created))
        .send()
        .await
        .unwrap();

    assert_eq!(tickets.status(), StatusCode::UNAUTHORIZED);
    assert_eq!(keys.status(), StatusCode::UNAUTHORIZED);
    assert_eq!(revocation.status(), StatusCode::UNAUTHORIZED);
    assert_eq!(last_used_at(&app, key_id(&created)).await, None);
    assert_eq!(revoked_at(&app, key_id(&created)).await, None);
    app.cleanup().await;
}
