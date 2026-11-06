use std::{
    collections::HashMap,
    net::SocketAddr,
    sync::{
        Arc, Mutex,
        atomic::{AtomicUsize, Ordering},
    },
};

use axum::{
    Form, Json, Router,
    extract::State,
    http::HeaderMap,
    routing::{get, post},
};
use serde_json::{Value, json};
use url::Url;
use uuid::Uuid;

use crate::{client, harness};

#[derive(Default)]
struct Provider {
    exchanges: Mutex<Vec<HashMap<String, String>>>,
    refreshes: AtomicUsize,
}

async fn token(
    State(provider): State<Arc<Provider>>,
    Form(form): Form<HashMap<String, String>>,
) -> (axum::http::StatusCode, Json<Value>) {
    provider.exchanges.lock().unwrap().push(form.clone());
    match form.get("grant_type").map(String::as_str) {
        Some("authorization_code") if form.get("code").map(String::as_str) == Some("good-code") => {
            (
                axum::http::StatusCode::OK,
                Json(
                    json!({ "access_token": "AT-SECRET", "refresh_token": "RT-SECRET", "expires_in": 3600 }),
                ),
            )
        }
        Some("refresh_token") => {
            provider.refreshes.fetch_add(1, Ordering::SeqCst);
            (
                axum::http::StatusCode::OK,
                Json(json!({ "access_token": "AT-REFRESHED", "expires_in": 3600 })),
            )
        }
        _ => (
            axum::http::StatusCode::BAD_REQUEST,
            Json(json!({ "error": "invalid_grant" })),
        ),
    }
}

async fn userinfo(headers: HeaderMap) -> (axum::http::StatusCode, Json<Value>) {
    let bearer = headers
        .get("authorization")
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default();
    if bearer == "Bearer AT-SECRET" {
        (
            axum::http::StatusCode::OK,
            Json(json!({ "email": "ada@example.com" })),
        )
    } else {
        (axum::http::StatusCode::UNAUTHORIZED, Json(json!({})))
    }
}

async fn fake_provider() -> (String, Arc<Provider>) {
    let provider = Arc::new(Provider::default());
    let router = Router::new()
        .route("/token", post(token))
        .route("/userinfo", get(userinfo))
        .with_state(provider.clone());
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address: SocketAddr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        let _ = axum::serve(listener, router).await;
    });
    (format!("http://{address}"), provider)
}

fn no_redirects() -> reqwest::Client {
    reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .unwrap()
}

async fn create_oauth_credential(app: &harness::App, provider: &str, name: &str) -> String {
    let created: Value = client()
        .post(app.credentials_url())
        .bearer_auth(&app.token)
        .json(&json!({
            "kind": "oauth2_authorization_code",
            "name": name,
            "origin": "supplied",
            "data": {
                "authorization_url": format!("{provider}/authorize"),
                "token_url": format!("{provider}/token"),
                "userinfo_url": format!("{provider}/userinfo"),
                "client_id": "mestier",
                "client_secret": "CLIENT-SECRET",
                "scopes": "openid",
            }
        }))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    created["data"]["id"].as_str().expect("an id").to_owned()
}

async fn start(app: &harness::App, token: &str, id: &str, body: Value) -> reqwest::Response {
    client()
        .post(format!("{}/oauth/start", app.credential_url(id)))
        .bearer_auth(token)
        .json(&body)
        .send()
        .await
        .unwrap()
}

fn start_body(app: &harness::App) -> Value {
    json!({
        "redirect_uri": format!("{}/oauth/callback", app.base_url),
        "return_to": "/o/atelier/automatisation/credentials",
    })
}

async fn authorization_url(app: &harness::App, id: &str) -> String {
    let response = start(app, &app.token, id, start_body(app)).await;
    assert_eq!(response.status(), 200, "{}", response.text().await.unwrap());
    response.json::<Value>().await.unwrap()["data"]["authorization_url"]
        .as_str()
        .expect("an authorization url")
        .to_owned()
}

fn state_of(url: &str) -> String {
    Url::parse(url)
        .unwrap()
        .query_pairs()
        .find(|(key, _)| key == "state")
        .map(|(_, value)| value.into_owned())
        .expect("a state")
}

async fn callback(app: &harness::App, query: &str) -> reqwest::Response {
    no_redirects()
        .get(format!("{}/oauth/callback?{query}", app.base_url))
        .send()
        .await
        .unwrap()
}

async fn credential_row(app: &harness::App, id: &str) -> Value {
    client()
        .get(app.credentials_url())
        .bearer_auth(&app.token)
        .send()
        .await
        .unwrap()
        .json::<Value>()
        .await
        .unwrap()["data"]
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row["id"] == id)
        .expect("the credential is listed")
        .clone()
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn an_authorization_code_flow_connects_an_account_and_a_replay_is_refused() {
    let (provider_url, provider) = fake_provider().await;
    let app = harness::start_with_oauth_client().await;
    let id = create_oauth_credential(&app, &provider_url, "Compte").await;
    assert_eq!(credential_row(&app, &id).await["status"], "not_connected");

    let url = authorization_url(&app, &id).await;
    assert!(
        url.starts_with(&format!("{provider_url}/authorize?")),
        "{url}"
    );
    let state = state_of(&url);
    assert!(state.len() >= 43);

    let done = callback(&app, &format!("code=good-code&state={state}")).await;

    assert_eq!(done.status(), 303);
    assert_eq!(
        done.headers()["location"],
        "/o/atelier/automatisation/credentials?oauth=ok"
    );
    assert_eq!(done.headers()["cache-control"], "no-store");
    let row = credential_row(&app, &id).await;
    assert_eq!(row["status"], "valid");
    assert_eq!(row["account"], "ada@example.com");
    assert!(row["expires_at"].is_string());
    assert!(
        row["secrets_set"]
            .as_array()
            .unwrap()
            .iter()
            .any(|name| name == "refresh_token")
    );
    let listing = serde_json::to_string(&row).unwrap();
    assert!(
        !listing.contains("AT-SECRET")
            && !listing.contains("RT-SECRET")
            && !listing.contains("CLIENT-SECRET")
    );
    let exchanged = provider.exchanges.lock().unwrap()[0].clone();
    assert_eq!(exchanged["code"], "good-code");
    assert!(exchanged.contains_key("code_verifier"));
    assert!(!exchanged.values().any(|value| *value == state));

    let replay = callback(&app, &format!("code=good-code&state={state}")).await;
    assert_eq!(replay.status(), 400);
    assert_eq!(provider.exchanges.lock().unwrap().len(), 1);

    let disconnected: Value = client()
        .post(format!("{}/oauth/disconnect", app.credential_url(&id)))
        .bearer_auth(&app.token)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(disconnected["data"]["status"], "not_connected");
    assert_eq!(disconnected["data"]["account"], Value::Null);
    assert_eq!(
        disconnected["data"]["secrets_set"],
        json!(["client_secret"])
    );

    app.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn an_expired_unknown_or_missing_state_is_refused_and_a_rejected_code_redirects_as_an_error()
{
    let (provider_url, provider) = fake_provider().await;
    let app = harness::start_with_oauth_client().await;
    let id = create_oauth_credential(&app, &provider_url, "Compte").await;
    let state = state_of(&authorization_url(&app, &id).await);
    sqlx::query(
        "UPDATE automation.oauth_state
         SET created_at = now() - interval '11 minutes', expires_at = now() - interval '1 minute'",
    )
    .execute(&app.pool)
    .await
    .unwrap();

    for query in [
        format!("code=good-code&state={state}"),
        "code=good-code&state=never-issued".to_owned(),
        "code=good-code".to_owned(),
        "state=".to_owned(),
    ] {
        let refused = callback(&app, &query).await;

        assert_eq!(refused.status(), 400, "{query}");
        assert_eq!(refused.headers()["cache-control"], "no-store");
    }
    assert!(provider.exchanges.lock().unwrap().is_empty());

    let state = state_of(&authorization_url(&app, &id).await);
    let rejected = callback(&app, &format!("code=bad-code&state={state}")).await;
    assert_eq!(rejected.status(), 303);
    assert_eq!(
        rejected.headers()["location"],
        "/o/atelier/automatisation/credentials?oauth=error"
    );
    let state = state_of(&authorization_url(&app, &id).await);
    let denied = callback(&app, &format!("error=access_denied&state={state}")).await;
    assert_eq!(
        denied.headers()["location"],
        "/o/atelier/automatisation/credentials?oauth=error"
    );
    assert_eq!(credential_row(&app, &id).await["status"], "not_connected");

    app.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn starting_needs_manage_rights_in_this_organization_a_connectable_kind_and_honest_addresses()
{
    let (provider_url, _) = fake_provider().await;
    let app = harness::start_with_oauth_client().await;
    let id = create_oauth_credential(&app, &provider_url, "Compte").await;

    let view_only = start(&app, &app.view_only_token, &id, start_body(&app)).await;
    assert_eq!(view_only.status(), 403);
    let outsider = start(&app, &app.outsider_token, &id, start_body(&app)).await;
    assert_eq!(outsider.status(), 403);

    let bearer: Value = client()
        .post(app.credentials_url())
        .bearer_auth(&app.token)
        .json(&json!({ "kind": "bearer_token", "name": "Jeton", "origin": "supplied", "data": { "token": "t" } }))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let not_connectable = start(
        &app,
        &app.token,
        bearer["data"]["id"].as_str().unwrap(),
        start_body(&app),
    )
    .await;
    assert_eq!(not_connectable.status(), 400);
    assert_eq!(
        not_connectable.json::<Value>().await.unwrap()["code"],
        "E_VALIDATION"
    );

    for body in [
        json!({ "redirect_uri": "https://evil.example/elsewhere", "return_to": "/ok" }),
        json!({ "redirect_uri": format!("{}/oauth/callback", app.base_url), "return_to": "https://evil.example/" }),
        json!({ "redirect_uri": format!("{}/oauth/callback", app.base_url), "return_to": "//evil.example/" }),
    ] {
        let refused = start(&app, &app.token, &id, body.clone()).await;

        assert_eq!(refused.status(), 400, "{body}");
    }
    let missing = start(
        &app,
        &app.token,
        &Uuid::now_v7().to_string(),
        start_body(&app),
    )
    .await;
    assert_eq!(missing.status(), 404);

    app.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn without_a_public_api_url_only_a_localhost_redirect_is_accepted_and_the_dev_origin_is_trusted()
 {
    let (provider_url, _) = fake_provider().await;
    let app = harness::start_with_oauth_client().await;
    let id = create_oauth_credential(&app, &provider_url, "Compte").await;

    let foreign = start(
        &app,
        &app.token,
        &id,
        json!({ "redirect_uri": "https://api.mestier.fr/oauth/callback", "return_to": "/ok" }),
    )
    .await;
    let local = start(
        &app,
        &app.token,
        &id,
        json!({
            "redirect_uri": format!("{}/oauth/callback", app.base_url),
            "return_to": "http://localhost:3000/o/atelier/automatisation/credentials",
        }),
    )
    .await;

    assert_eq!(foreign.status(), 409);
    let refusal = foreign.json::<Value>().await.unwrap();
    assert_eq!(refusal["code"], "E_OAUTH_NOT_CONFIGURED");
    assert_eq!(refusal["status"], 409);
    assert!(
        !refusal["message"]
            .as_str()
            .unwrap()
            .contains("api.mestier.fr")
    );
    assert_eq!(local.status(), 200);

    app.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn with_a_public_api_url_only_its_callback_is_accepted_and_the_dev_origin_is_not() {
    let (provider_url, _) = fake_provider().await;
    let app = harness::start_with_public_api_url("https://api.mestier.test/").await;
    let id = create_oauth_credential(&app, &provider_url, "Compte").await;
    let attempt = |redirect_uri: String, return_to: &str| {
        let body = json!({ "redirect_uri": redirect_uri, "return_to": return_to });
        let app = &app;
        let id = id.clone();
        async move { start(app, &app.token, &id, body).await }
    };

    let configured = attempt("https://api.mestier.test/oauth/callback".to_owned(), "/ok").await;
    let local_callback = attempt(format!("{}/oauth/callback", app.base_url), "/ok").await;
    let other_host = attempt("https://evil.example/oauth/callback".to_owned(), "/ok").await;
    let dev_origin = attempt(
        "https://api.mestier.test/oauth/callback".to_owned(),
        "http://localhost:3000/o/atelier",
    )
    .await;

    let allowed_origin = attempt(
        "https://api.mestier.test/oauth/callback".to_owned(),
        "https://app.mestier.test/o/atelier",
    )
    .await;

    assert_eq!(configured.status(), 200);
    assert_eq!(allowed_origin.status(), 200);
    for refused in [local_callback, other_host, dev_origin] {
        assert_eq!(refused.status(), 400);
        assert_eq!(
            refused.json::<Value>().await.unwrap()["code"],
            "E_VALIDATION"
        );
    }

    app.cleanup().await;
}
