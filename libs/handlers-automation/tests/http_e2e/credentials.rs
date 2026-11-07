use chrono::{Duration, Utc};
use mestier_core::{SealedSecret, SecretCipher};
use serde_json::{Value, json};
use uuid::Uuid;

use crate::{client, harness};

const KEY: &str = "MDEyMzQ1Njc4OWFiY2RlZjAxMjM0NTY3ODlhYmNkZWY=";

async fn create(app: &harness::App, body: Value) -> reqwest::Response {
    client()
        .post(app.credentials_url())
        .bearer_auth(&app.token)
        .json(&body)
        .send()
        .await
        .expect("the api answers the credential create call")
}

async fn created(app: &harness::App, body: Value) -> Value {
    let response = create(app, body).await;
    assert_eq!(response.status(), 201, "{}", response.text().await.unwrap());
    response.json::<Value>().await.expect("json")["data"].clone()
}

async fn patch(app: &harness::App, id: &str, body: Value) -> reqwest::Response {
    client()
        .patch(app.credential_url(id))
        .bearer_auth(&app.token)
        .json(&body)
        .send()
        .await
        .expect("the api answers the credential update call")
}

async fn listed(app: &harness::App) -> Vec<Value> {
    client()
        .get(app.credentials_url())
        .bearer_auth(&app.token)
        .send()
        .await
        .expect("the api answers the credential list call")
        .json::<Value>()
        .await
        .expect("json")["data"]
        .as_array()
        .expect("a list")
        .clone()
}

async fn sealed_fields(app: &harness::App, id: &str) -> Value {
    let row: (Vec<u8>, Vec<u8>) = sqlx::query_as(
        "SELECT data_nonce, data_ciphertext FROM automation.credential WHERE id = $1",
    )
    .bind(id.parse::<Uuid>().expect("a uuid"))
    .fetch_one(&app.pool)
    .await
    .expect("the credential row");
    let cipher = SecretCipher::from_base64(KEY).expect("the test key");
    let plaintext = cipher
        .open(&SealedSecret {
            nonce: row.0,
            ciphertext: row.1,
        })
        .expect("the sealed bytes open");
    serde_json::from_slice(&plaintext).expect("a json object")
}

fn ssh_body(name: &str) -> Value {
    json!({
        "kind": "ssh",
        "name": name,
        "description": "Serveur de fichiers",
        "origin": "supplied",
        "data": {
            "host": "srv.example.com", "port": 22, "username": "root",
            "method": "private-key", "private_key": "TOP-SECRET-KEY", "passphrase": "TOP-SECRET-PHRASE",
        }
    })
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn a_created_credential_returns_readable_values_and_the_presence_of_each_secret_never_a_secret()
 {
    let app = harness::start().await;

    let response = create(&app, ssh_body("Serveur")).await;
    assert_eq!(response.status(), 201);
    let text = response.text().await.unwrap();

    assert!(!text.contains("TOP-SECRET"), "{text}");
    let body: Value = serde_json::from_str(&text).unwrap();
    let credential = &body["data"];
    assert_eq!(credential["description"], "Serveur de fichiers");
    assert_eq!(credential["status"], "valid");
    assert_eq!(credential["account"], Value::Null);
    assert_eq!(credential["expires_at"], Value::Null);
    assert_eq!(credential["used_by_workflows"], 0);
    assert_eq!(credential["origin"], "supplied");
    assert_eq!(
        credential["data"],
        json!({ "host": "srv.example.com", "port": 22, "username": "root", "method": "private-key" })
    );
    assert_eq!(
        credential["secrets_set"],
        json!(["passphrase", "private_key"])
    );
    assert_eq!(credential["secret"], Value::Null);
    let mut keys: Vec<&str> = credential
        .as_object()
        .unwrap()
        .keys()
        .map(String::as_str)
        .collect();
    keys.sort_unstable();
    assert_eq!(
        keys,
        [
            "account",
            "created_at",
            "data",
            "description",
            "expires_at",
            "id",
            "kind",
            "name",
            "organization_id",
            "origin",
            "secret",
            "secrets_set",
            "status",
            "updated_at",
            "used_by_workflows",
        ]
    );

    let listing = client()
        .get(app.credentials_url())
        .bearer_auth(&app.token)
        .send()
        .await
        .unwrap()
        .text()
        .await
        .unwrap();
    assert!(!listing.contains("TOP-SECRET"), "{listing}");

    app.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn editing_with_a_single_secret_keeps_the_other_secrets_and_the_readable_values() {
    let app = harness::start().await;
    let credential = created(&app, ssh_body("Serveur")).await;
    let id = credential["id"].as_str().unwrap();

    let response = patch(&app, id, json!({ "data": { "passphrase": "NEW-PHRASE" } })).await;

    assert_eq!(response.status(), 200);
    let text = response.text().await.unwrap();
    assert!(
        !text.contains("NEW-PHRASE") && !text.contains("TOP-SECRET"),
        "{text}"
    );
    let body: Value = serde_json::from_str(&text).unwrap();
    assert_eq!(
        body["data"]["secrets_set"],
        json!(["passphrase", "private_key"])
    );
    assert_eq!(body["data"]["data"]["host"], "srv.example.com");
    let sealed = sealed_fields(&app, id).await;
    assert_eq!(
        sealed,
        json!({ "private_key": "TOP-SECRET-KEY", "passphrase": "NEW-PHRASE" })
    );

    app.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn an_explicit_null_secret_is_refused_in_french_and_changes_nothing() {
    let app = harness::start().await;
    let credential = created(&app, ssh_body("Serveur")).await;
    let id = credential["id"].as_str().unwrap();

    let response = patch(&app, id, json!({ "data": { "passphrase": null } })).await;

    assert_eq!(response.status(), 400);
    let body: Value = response.json().await.unwrap();
    assert_eq!(body["code"], "E_VALIDATION");
    assert!(
        body["message"]
            .as_str()
            .unwrap()
            .contains("Le champ « Phrase de passe » ne peut pas valoir null"),
        "{body}"
    );
    assert_eq!(
        sealed_fields(&app, id).await["passphrase"],
        "TOP-SECRET-PHRASE"
    );

    app.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn the_catalogue_of_kinds_carries_french_labels_categories_usages_defaults_and_hints() {
    let app = harness::start().await;

    let response = client()
        .get(app.credential_kinds_url())
        .bearer_auth(&app.view_only_token)
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), 200);
    let kinds = response.json::<Value>().await.unwrap()["data"]
        .as_array()
        .unwrap()
        .clone();
    assert_eq!(kinds.len(), 18);
    let kind = |code: &str| kinds.iter().find(|kind| kind["kind"] == code).unwrap();
    assert_eq!(kind("http_basic")["label"], "Basic Auth");
    assert_eq!(kind("http_header")["label"], "Clé d’API dans un en-tête");
    assert_eq!(kind("http_header")["category"], "http");
    assert_eq!(kind("hmac")["category"], "webhook");
    assert_eq!(kind("smtp")["category"], "email");
    assert_eq!(kind("smtp")["category_label"], "E-mail");
    assert_eq!(kind("ssh")["category"], "servers");
    assert_eq!(
        kind("http_basic")["usages"][0]["label"],
        "Requête HTTP et opérations de collection"
    );
    assert_eq!(kind("http_basic")["usages"][1]["usage"], "webhook");
    let http_header_fields = kind("http_header")["fields"].as_array().unwrap();
    assert_eq!(http_header_fields[0]["default"], "X-API-Key");
    assert_eq!(
        kind("oauth2_client_credentials")["fields"][3]["hint"],
        "Séparés par des espaces."
    );
    assert_eq!(kind("smtp")["fields"][0]["placeholder"], "smtp.exemple.fr");
    assert_eq!(kind("imap")["executable"], false);
    assert_eq!(kind("smtp")["executable"], true);
    assert_eq!(
        kind("bearer_token")["origin_choices"],
        json!(["supplied", "generated"])
    );
    let password = kind("ssh")["fields"]
        .as_array()
        .unwrap()
        .iter()
        .find(|field| field["name"] == "password")
        .unwrap();
    assert_eq!(
        password["show_when"],
        json!({ "field": "method", "one_of": ["password"] })
    );

    let forbidden = client()
        .get(app.credential_kinds_url())
        .bearer_auth(&app.no_role_token)
        .send()
        .await
        .unwrap();
    assert_eq!(forbidden.status(), 403);

    app.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn every_kind_can_be_created_from_its_required_fields() {
    let app = harness::start().await;
    let samples = [
        ("http_basic", json!({ "username": "a", "password": "p" })),
        ("bearer_token", json!({ "token": "t" })),
        (
            "http_header",
            json!({ "header_name": "X-K", "header_value": "v" }),
        ),
        (
            "api_key_query",
            json!({ "param_name": "key", "param_value": "v" }),
        ),
        ("digest", json!({ "username": "a", "password": "p" })),
        (
            "oauth2_authorization_code",
            json!({
                "authorization_url": "https://a.example.com/auth",
                "token_url": "https://a.example.com/token",
                "client_id": "id", "client_secret": "s",
            }),
        ),
        (
            "oauth2_client_credentials",
            json!({ "token_url": "https://a.example.com/token", "client_id": "id", "client_secret": "s" }),
        ),
        (
            "oauth1",
            json!({
                "consumer_key": "k", "consumer_secret": "s",
                "request_token_url": "https://a.example.com/rt",
                "authorization_url": "https://a.example.com/au",
                "access_token_url": "https://a.example.com/at",
            }),
        ),
        ("jwt", json!({ "key": "shared-secret" })),
        (
            "mtls",
            json!({ "certificate": "-----BEGIN CERTIFICATE-----", "private_key": "-----BEGIN PRIVATE KEY-----" }),
        ),
        (
            "custom_auth",
            json!({ "json": { "headers": { "X-Token": "t" } } }),
        ),
        ("hmac", json!({ "header_name": "X-Sig", "secret": "s" })),
        ("smtp", json!({ "host": "smtp.example.com", "port": 465 })),
        (
            "imap",
            json!({ "host": "imap.example.com", "port": 993, "username": "u", "password": "p" }),
        ),
        (
            "ssh",
            json!({ "host": "h", "port": 22, "username": "u", "private_key": "KEY" }),
        ),
        (
            "ftp",
            json!({ "host": "h", "port": 22, "username": "u", "password": "p" }),
        ),
        (
            "sql",
            json!({ "host": "h", "port": 5432, "database": "d", "username": "u", "password": "p" }),
        ),
        (
            "odoo_api",
            json!({ "base_url": "https://o.example.com", "database": "d", "username": "u", "api_key": "k" }),
        ),
    ];

    for (kind, data) in samples {
        let credential = created(
            &app,
            json!({ "kind": kind, "name": format!("Test {kind}"), "origin": "supplied", "data": data }),
        )
        .await;
        let expected_status = if matches!(kind, "oauth2_authorization_code" | "oauth1") {
            "not_connected"
        } else {
            "valid"
        };
        assert_eq!(credential["status"], expected_status, "{kind}");
    }

    app.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn a_field_that_does_not_match_the_kind_is_a_french_validation_error_without_a_constraint_name()
 {
    let app = harness::start().await;

    for (body, expected) in [
        (
            json!({ "kind": "smtp", "name": "S", "origin": "supplied", "data": { "port": 25 } }),
            "Le champ « Hôte » est obligatoire.",
        ),
        (
            json!({ "kind": "smtp", "name": "S", "origin": "supplied", "data": { "host": "h", "port": "abc" } }),
            "Le champ « Port » doit être un nombre entier.",
        ),
        (
            json!({ "kind": "smtp", "name": "S", "origin": "supplied", "data": { "host": "h", "port": 25, "typo": 1 } }),
            "Le champ « typo » n’existe pas pour ce type d’identification.",
        ),
        (
            json!({ "kind": "custom_auth", "name": "C", "origin": "supplied", "data": { "json": "pas du json" } }),
            "JSON invalide : un objet est attendu.",
        ),
        (
            json!({ "kind": "nope", "name": "C", "origin": "supplied", "data": {} }),
            "Le type d’identification « nope » n’existe pas.",
        ),
        (
            json!({ "kind": "bearer_token", "name": "  ", "origin": "supplied", "data": { "token": "t" } }),
            "Donnez un nom à cette identification.",
        ),
    ] {
        let response = create(&app, body.clone()).await;

        assert_eq!(response.status(), 400, "{body}");
        let answer: Value = response.json().await.unwrap();
        assert_eq!(answer["code"], "E_VALIDATION", "{body}");
        assert!(
            answer["message"].as_str().unwrap().contains(expected),
            "{answer}"
        );
    }

    app.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn a_name_used_twice_for_a_kind_answers_a_static_conflict() {
    let app = harness::start().await;
    created(&app, ssh_body("Serveur")).await;

    let response = create(&app, ssh_body("Serveur")).await;

    assert_eq!(response.status(), 409);
    let answer: Value = response.json().await.unwrap();
    assert_eq!(answer["code"], "E_CONFLICT");
    let message = answer["message"].as_str().unwrap();
    assert!(
        message.contains("Une identification de ce type porte déjà ce nom."),
        "{message}"
    );
    assert!(!message.contains("uq_"), "{message}");

    app.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn a_credential_past_its_expiry_is_listed_as_expired_and_the_list_counts_the_workflows_using_it()
 {
    let app = harness::start().await;
    let credential = created(
        &app,
        json!({
            "kind": "bearer_token", "name": "Jeton", "origin": "supplied",
            "data": { "token": "abc" },
            "expires_at": (Utc::now() - Duration::days(1)).to_rfc3339(),
        }),
    )
    .await;
    let id = credential["id"].as_str().unwrap().to_owned();
    let other = created(
        &app,
        json!({ "kind": "bearer_token", "name": "Autre", "origin": "supplied", "data": { "token": "abc" } }),
    )
    .await;

    let workflow: Value = client()
        .post(app.workflows_url())
        .bearer_auth(&app.token)
        .json(&json!({ "name": "Appelle l’API" }))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let workflow_id = workflow["data"]["id"].as_str().unwrap();
    let saved = client()
        .put(app.workflow_versions_url(workflow_id))
        .bearer_auth(&app.token)
        .json(&json!({
            "graph": {
                "connectors": [{
                    "id": "c1", "kind": "http.request", "version": 1, "credential_id": id,
                    "config": { "method": "GET", "url": "https://example.com/" }
                }],
                "edges": [{ "from": "t1", "to": "c1", "branch": null }],
                "triggers": [{ "id": "t1", "kind": "Manual" }]
            },
            "layout": null
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(saved.status(), 201);

    let rows = listed(&app).await;
    let find = |wanted: &str| rows.iter().find(|row| row["id"] == wanted).unwrap().clone();

    assert_eq!(find(&id)["status"], "expired");
    assert_eq!(find(&id)["used_by_workflows"], 1);
    assert_eq!(find(other["id"].as_str().unwrap())["status"], "valid");
    assert_eq!(find(other["id"].as_str().unwrap())["used_by_workflows"], 0);

    let renewed = patch(
        &app,
        &id,
        json!({ "expires_at": (Utc::now() + Duration::days(30)).to_rfc3339() }),
    )
    .await;
    assert_eq!(renewed.status(), 200);
    let renewed: Value = renewed.json().await.unwrap();
    assert_eq!(renewed["data"]["status"], "valid");
    assert_eq!(renewed["data"]["used_by_workflows"], 1);

    app.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn deleting_a_used_credential_succeeds_and_its_workflow_keeps_the_dangling_reference() {
    let app = harness::start().await;
    let credential = created(
        &app,
        json!({ "kind": "bearer_token", "name": "Jeton", "origin": "supplied", "data": { "token": "abc" } }),
    )
    .await;
    let id = credential["id"].as_str().unwrap().to_owned();
    let workflow: Value = client()
        .post(app.workflows_url())
        .bearer_auth(&app.token)
        .json(&json!({ "name": "Appelle l’API" }))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let workflow_id = workflow["data"]["id"].as_str().unwrap();
    let graph = json!({
        "graph": {
            "connectors": [{
                "id": "c1", "kind": "http.request", "version": 1, "credential_id": id,
                "config": { "method": "GET", "url": "https://example.com/" }
            }],
            "edges": [{ "from": "t1", "to": "c1", "branch": null }],
            "triggers": [{ "id": "t1", "kind": "Manual" }]
        },
        "layout": null
    });
    let save = || async {
        client()
            .put(app.workflow_versions_url(workflow_id))
            .bearer_auth(&app.token)
            .json(&graph)
            .send()
            .await
            .unwrap()
    };
    assert_eq!(save().await.status(), 201);

    let deleted = client()
        .delete(app.credential_url(&id))
        .bearer_auth(&app.token)
        .send()
        .await
        .unwrap();

    assert_eq!(deleted.status(), 204);
    assert!(listed(&app).await.is_empty());
    assert_eq!(
        save().await.status(),
        422,
        "saving a graph naming an unknown credential is still refused"
    );

    app.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn a_credential_of_another_organization_is_never_listed_read_or_changed() {
    let app = harness::start().await;
    let mine = created(&app, ssh_body("Serveur")).await;
    let id = mine["id"].as_str().unwrap();
    let foreign = client()
        .post(app.other_organization_credentials_url())
        .bearer_auth(&app.outsider_token)
        .json(&ssh_body("Chez l’autre"))
        .send()
        .await
        .unwrap();
    assert_eq!(foreign.status(), 201);

    let mine_listed = listed(&app).await;
    assert_eq!(mine_listed.len(), 1);
    assert_eq!(mine_listed[0]["name"], "Serveur");

    let outsider_url = format!("{}/{id}", app.other_organization_credentials_url());
    let renamed = client()
        .patch(&outsider_url)
        .bearer_auth(&app.outsider_token)
        .json(&json!({ "name": "Volé" }))
        .send()
        .await
        .unwrap();
    assert_eq!(renamed.status(), 404);
    let deleted = client()
        .delete(&outsider_url)
        .bearer_auth(&app.outsider_token)
        .send()
        .await
        .unwrap();
    assert_eq!(deleted.status(), 404);
    let cross = client()
        .patch(app.credential_url(id))
        .bearer_auth(&app.outsider_token)
        .json(&json!({ "name": "Volé" }))
        .send()
        .await
        .unwrap();
    assert_eq!(cross.status(), 403);
    assert_eq!(listed(&app).await[0]["name"], "Serveur");

    sqlx::query("DELETE FROM automation.credential WHERE org_id = $1")
        .bind(app.other_organization_id())
        .execute(&app.pool)
        .await
        .unwrap();
    app.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn a_generated_credential_still_reveals_its_secret_once_on_creation_and_rotation() {
    let app = harness::start().await;

    let response = create(
        &app,
        json!({ "kind": "bearer_token", "name": "Signature", "origin": "generated" }),
    )
    .await;
    assert_eq!(response.status(), 201);
    let body: Value = response.json().await.unwrap();
    let first = body["data"]["secret"].as_str().expect("a base64 secret");
    assert!(!first.is_empty());
    assert_eq!(body["data"]["origin"], "generated");
    assert_eq!(body["data"]["secrets_set"], json!(["token"]));

    let id = body["data"]["id"].as_str().unwrap();
    let rotated: Value = client()
        .post(format!("{}/rotate", app.credential_url(id)))
        .bearer_auth(&app.token)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let second = rotated["data"]["secret"].as_str().expect("a fresh secret");
    assert_ne!(first, second);

    let listing = client()
        .get(app.credentials_url())
        .bearer_auth(&app.token)
        .send()
        .await
        .unwrap()
        .text()
        .await
        .unwrap();
    assert!(!listing.contains(first) && !listing.contains(second));

    app.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn a_view_only_member_lists_credentials_but_cannot_create_or_change_them() {
    let app = harness::start().await;
    let credential = created(&app, ssh_body("Serveur")).await;

    let listing = client()
        .get(app.credentials_url())
        .bearer_auth(&app.view_only_token)
        .send()
        .await
        .unwrap();
    assert_eq!(listing.status(), 200);
    let attempt = client()
        .post(app.credentials_url())
        .bearer_auth(&app.view_only_token)
        .json(&ssh_body("Autre"))
        .send()
        .await
        .unwrap();
    assert_eq!(attempt.status(), 403);
    let change = client()
        .patch(app.credential_url(credential["id"].as_str().unwrap()))
        .bearer_auth(&app.view_only_token)
        .json(&json!({ "name": "Volé" }))
        .send()
        .await
        .unwrap();
    assert_eq!(change.status(), 403);

    app.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn an_http_node_saves_with_every_http_kind_and_is_refused_a_kind_no_call_consumes() {
    let app = harness::start().await;
    let workflow: Value = client()
        .post(app.workflows_url())
        .bearer_auth(&app.token)
        .json(&json!({ "name": "Appelle l’API" }))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let workflow_id = workflow["data"]["id"].as_str().unwrap().to_owned();
    let save = |credential_id: String| {
        let request = client()
            .put(app.workflow_versions_url(&workflow_id))
            .bearer_auth(&app.token)
            .json(&json!({
                "graph": {
                    "connectors": [{
                        "id": "c1", "kind": "http.request", "version": 1,
                        "credential_id": credential_id,
                        "config": { "method": "GET", "url": "https://example.com/" }
                    }],
                    "edges": [{ "from": "t1", "to": "c1", "branch": null }],
                    "triggers": [{ "id": "t1", "kind": "Manual" }]
                },
                "layout": null
            }));
        async move { request.send().await.unwrap() }
    };

    for (kind, data) in [
        (
            "api_key_query",
            json!({ "param_name": "k", "param_value": "v" }),
        ),
        ("digest", json!({ "username": "a", "password": "p" })),
        ("jwt", json!({ "key": "shared" })),
        (
            "custom_auth",
            json!({ "json": { "headers": { "X-T": "t" } } }),
        ),
        (
            "oauth2_client_credentials",
            json!({ "token_url": "https://a.example.com/token", "client_id": "i", "client_secret": "s" }),
        ),
    ] {
        let credential = created(
            &app,
            json!({ "kind": kind, "name": format!("Test {kind}"), "origin": "supplied", "data": data }),
        )
        .await;

        let saved = save(credential["id"].as_str().unwrap().to_owned()).await;

        assert_eq!(saved.status(), 201, "{kind}");
    }

    for (kind, data) in [
        ("smtp", json!({ "host": "smtp.example.com", "port": 465 })),
        ("hmac", json!({ "header_name": "X-Sig", "secret": "s" })),
        (
            "ssh",
            json!({ "host": "h", "port": 22, "username": "u", "private_key": "K" }),
        ),
    ] {
        let credential = created(
            &app,
            json!({ "kind": kind, "name": format!("Test {kind}"), "origin": "supplied", "data": data }),
        )
        .await;

        let saved = save(credential["id"].as_str().unwrap().to_owned()).await;

        assert_eq!(saved.status(), 422, "{kind}");
    }

    app.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn every_kind_is_supported_and_only_the_executable_ones_say_a_node_can_use_them() {
    let app = harness::start().await;

    let kinds = client()
        .get(app.credential_kinds_url())
        .bearer_auth(&app.view_only_token)
        .send()
        .await
        .unwrap()
        .json::<Value>()
        .await
        .unwrap()["data"]
        .as_array()
        .unwrap()
        .clone();

    assert!(kinds.iter().all(|kind| kind["supported"] == json!(true)));
    let ssh = kinds.iter().find(|kind| kind["kind"] == "ssh").unwrap();
    assert_eq!(ssh["executable"], json!(false));
    let jwt = kinds.iter().find(|kind| kind["kind"] == "jwt").unwrap();
    assert_eq!(jwt["usages"].as_array().unwrap().len(), 1);
    assert!(!jwt["description"].as_str().unwrap().contains("vérifie"));
    let mtls = kinds.iter().find(|kind| kind["kind"] == "mtls").unwrap();
    let passphrase = mtls["fields"]
        .as_array()
        .unwrap()
        .iter()
        .find(|field| field["name"] == "passphrase")
        .unwrap();
    assert!(passphrase["hint"].as_str().unwrap().contains("chiffrées"));

    app.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn a_kind_that_cannot_be_generated_and_oversized_or_nul_values_are_validation_errors() {
    let app = harness::start().await;

    let generated = create(
        &app,
        json!({ "kind": "hmac", "name": "Signature", "origin": "generated" }),
    )
    .await;
    let long_name = create(
        &app,
        json!({ "kind": "bearer_token", "name": "n".repeat(121), "origin": "supplied", "data": { "token": "t" } }),
    )
    .await;
    let nul_secret = create(
        &app,
        json!({ "kind": "bearer_token", "name": "Nul", "origin": "supplied", "data": { "token": "a\u{0}b" } }),
    )
    .await;
    let huge = create(
        &app,
        json!({ "kind": "bearer_token", "name": "Huge", "origin": "supplied", "data": { "token": "x".repeat(9000) } }),
    )
    .await;

    for refused in [generated, long_name, nul_secret, huge] {
        assert_eq!(refused.status(), 400);
        assert_eq!(
            refused.json::<Value>().await.unwrap()["code"],
            "E_VALIDATION"
        );
    }

    app.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn moving_a_provider_address_needs_the_secret_again_and_leaves_the_account_disconnected() {
    let app = harness::start().await;
    let credential = created(
        &app,
        json!({
            "kind": "oauth2_authorization_code",
            "name": "Compte",
            "origin": "supplied",
            "data": {
                "authorization_url": "https://auth.example.com/authorize",
                "token_url": "https://auth.example.com/token",
                "client_id": "id",
                "client_secret": "CLIENT-SECRET-VALUE",
            }
        }),
    )
    .await;
    let id = credential["id"].as_str().unwrap().to_owned();

    let scopes = patch(&app, &id, json!({ "data": { "scopes": "openid" } })).await;
    let without_secret = patch(
        &app,
        &id,
        json!({ "data": { "token_url": "https://evil.example/token" } }),
    )
    .await;
    let insecure = patch(
        &app,
        &id,
        json!({ "data": { "token_url": "http://auth.example.com/token", "client_secret": "s" } }),
    )
    .await;
    let moved = patch(
        &app,
        &id,
        json!({ "data": { "token_url": "https://other.example.com/token", "client_secret": "s" } }),
    )
    .await;

    assert_eq!(scopes.status(), 200);
    assert_eq!(without_secret.status(), 400);
    let refusal = without_secret.json::<Value>().await.unwrap();
    assert_eq!(refusal["code"], "E_VALIDATION");
    assert!(
        refusal["message"]
            .as_str()
            .unwrap()
            .contains("Client secret")
    );
    assert!(!refusal.to_string().contains("CLIENT-SECRET-VALUE"));
    assert_eq!(insecure.status(), 400);
    assert_eq!(moved.status(), 200);
    let moved = moved.json::<Value>().await.unwrap();
    assert_eq!(moved["data"]["status"], "not_connected");
    assert_eq!(
        moved["data"]["data"]["token_url"],
        "https://other.example.com/token"
    );

    app.cleanup().await;
}
