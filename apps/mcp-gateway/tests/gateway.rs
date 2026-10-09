mod keys;

use std::{
    collections::HashMap,
    convert::Infallible,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, AtomicUsize, Ordering},
    },
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use axum::{
    Form, Json, Router,
    extract::State,
    http::{HeaderMap, Method, StatusCode},
    response::{IntoResponse, Response, Sse, sse::Event},
    routing::{any, get, post},
};
use futures_util::stream;
use jsonwebtoken::{Algorithm, EncodingKey, Header, encode};
use mcp_gateway::{
    app::{AppState, Gateway, router},
    config::ClientAuthMethod,
    exchange::{CachedExchanger, FerrisKeyExchanger},
    pdp::AuthzenPdp,
    verifier::JwksVerifier,
};
use reqwest::Client;
use serde_json::{Value, json};
use tokio::{net::TcpListener, sync::Notify};

const RESOURCE: &str = "https://mcp.example/mcp";
const METADATA: &str = "https://mcp.example/.well-known/oauth-protected-resource";

async fn serve(app: Router) -> String {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
    format!("http://{addr}")
}

fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

#[derive(Default)]
struct AsState {
    exchanges: AtomicUsize,
    fail_exchange: AtomicBool,
    forms: Mutex<Vec<HashMap<String, String>>>,
}

async fn jwks() -> Json<Value> {
    Json(json!({"keys": [{
        "kty": "RSA", "alg": "RS256", "use": "sig",
        "kid": keys::KID, "n": keys::PUBLIC_N, "e": keys::PUBLIC_E,
    }]}))
}

async fn token_endpoint(
    State(state): State<Arc<AsState>>,
    Form(form): Form<HashMap<String, String>>,
) -> Response {
    state.forms.lock().unwrap().push(form.clone());
    match form.get("grant_type").map(String::as_str) {
        Some("client_credentials") => {
            Json(json!({"access_token": "actor-token", "expires_in": 300})).into_response()
        }
        Some("urn:ietf:params:oauth:grant-type:token-exchange") => {
            if state.fail_exchange.load(Ordering::SeqCst) {
                return StatusCode::BAD_REQUEST.into_response();
            }
            let n = state.exchanges.fetch_add(1, Ordering::SeqCst) + 1;
            Json(json!({
                "access_token": format!("delegated-{n}"),
                "issued_token_type": "urn:ietf:params:oauth:token-type:access_token",
                "token_type": "Bearer",
                "expires_in": 300,
            }))
            .into_response()
        }
        _ => StatusCode::BAD_REQUEST.into_response(),
    }
}

#[derive(Clone, Copy, PartialEq)]
enum PdpMode {
    Allow,
    Deny,
    Broken,
}

struct PdpState {
    mode: Mutex<PdpMode>,
    requests: Mutex<Vec<Value>>,
}

async fn evaluation(State(state): State<Arc<PdpState>>, Json(body): Json<Value>) -> Response {
    state.requests.lock().unwrap().push(body);
    match *state.mode.lock().unwrap() {
        PdpMode::Allow => Json(json!({"decision": true})).into_response(),
        PdpMode::Deny => Json(json!({"decision": false})).into_response(),
        PdpMode::Broken => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

#[derive(Default)]
struct UpstreamState {
    calls: Mutex<Vec<Recorded>>,
    release: Notify,
}

#[derive(Clone, Debug)]
struct Recorded {
    method: String,
    authorization: Option<String>,
    client: Option<String>,
    session: Option<String>,
}

async fn upstream(
    State(state): State<Arc<UpstreamState>>,
    method: Method,
    headers: HeaderMap,
) -> Response {
    let header = |name: &str| {
        headers
            .get(name)
            .and_then(|v| v.to_str().ok())
            .map(str::to_owned)
    };
    state.calls.lock().unwrap().push(Recorded {
        method: method.to_string(),
        authorization: header("authorization"),
        client: header("x-mestier-mcp-client"),
        session: header("mcp-session-id"),
    });
    if method == Method::GET {
        let events = stream::unfold((state, 0u8), |(state, step)| async move {
            match step {
                0 => Some((
                    Ok::<_, Infallible>(Event::default().data("first")),
                    (state, 1),
                )),
                1 => {
                    state.release.notified().await;
                    Some((Ok(Event::default().data("second")), (state, 2)))
                }
                _ => None,
            }
        });
        return Sse::new(events).into_response();
    }
    Json(json!({"jsonrpc": "2.0", "id": 1, "result": {"ok": true}})).into_response()
}

struct Harness {
    gateway: String,
    issuer: String,
    http: Client,
    auth: Arc<AsState>,
    pdp: Arc<PdpState>,
    upstream: Arc<UpstreamState>,
}

impl Harness {
    async fn start() -> Self {
        Self::start_with(None).await
    }

    async fn start_with(pdp_url: Option<String>) -> Self {
        let auth = Arc::new(AsState::default());
        let issuer_base = serve(
            Router::new()
                .route("/realms/mestier/protocol/openid-connect/certs", get(jwks))
                .route(
                    "/realms/mestier/protocol/openid-connect/token",
                    post(token_endpoint),
                )
                .with_state(Arc::clone(&auth)),
        )
        .await;
        let issuer = format!("{issuer_base}/realms/mestier");

        let pdp = Arc::new(PdpState {
            mode: Mutex::new(PdpMode::Allow),
            requests: Mutex::default(),
        });
        let pdp_base = serve(
            Router::new()
                .route("/access/v1/evaluation", post(evaluation))
                .with_state(Arc::clone(&pdp)),
        )
        .await;

        let up = Arc::new(UpstreamState::default());
        let upstream_base = serve(
            Router::new()
                .route("/mcp", any(upstream))
                .with_state(Arc::clone(&up)),
        )
        .await;

        let http = Client::new();
        let state = AppState::new(Gateway {
            verifier: JwksVerifier::new(http.clone(), &issuer, RESOURCE, Duration::from_secs(300))
                .with_unknown_kid_cooldown(Duration::ZERO),
            pdp: AuthzenPdp::new(http.clone(), &pdp_url.unwrap_or(pdp_base)),
            exchanger: CachedExchanger::new(FerrisKeyExchanger::new(
                http.clone(),
                &issuer,
                "mestier-mcp-gateway".into(),
                "s3cret".into(),
                "mestier-api".into(),
                ClientAuthMethod::Post,
            )),
            http: http.clone(),
            resource_url: RESOURCE.into(),
            issuer: issuer.clone(),
            upstream_url: format!("{upstream_base}/mcp"),
        });
        let gateway = serve(router(state)).await;
        Self {
            gateway,
            issuer,
            http,
            auth,
            pdp,
            upstream: up,
        }
    }

    fn token(&self, overrides: Value) -> String {
        let mut claims = json!({
            "iss": self.issuer(),
            "aud": RESOURCE,
            "sub": "alice",
            "azp": "chatgpt",
            "scope": "openid mestier.read",
            "exp": now() + 600,
        });
        for (k, v) in overrides.as_object().unwrap() {
            claims[k] = v.clone();
        }
        let mut header = Header::new(Algorithm::RS256);
        header.kid = Some(keys::KID.into());
        encode(
            &header,
            &claims,
            &EncodingKey::from_rsa_pem(keys::PRIVATE_KEY_PEM.as_bytes()).unwrap(),
        )
        .unwrap()
    }

    fn issuer(&self) -> String {
        self.issuer.clone()
    }

    async fn rpc(&self, token: Option<&str>, body: Value) -> reqwest::Response {
        let mut request = self
            .http
            .post(format!("{}/mcp", self.gateway))
            .header("content-type", "application/json")
            .header("accept", "application/json, text/event-stream")
            .json(&body);
        if let Some(token) = token {
            request = request.bearer_auth(token);
        }
        request.send().await.unwrap()
    }

    fn set_pdp(&self, mode: PdpMode) {
        *self.pdp.mode.lock().unwrap() = mode;
    }

    fn upstream_calls(&self) -> Vec<Recorded> {
        self.upstream.calls.lock().unwrap().clone()
    }
}

fn call(name: &str, organization: Option<&str>) -> Value {
    let mut arguments = json!({});
    if let Some(org) = organization {
        arguments["organization"] = json!(org);
    }
    json!({"jsonrpc": "2.0", "id": 9, "method": "tools/call",
        "params": {"name": name, "arguments": arguments}})
}

#[tokio::test]
async fn a_missing_token_gets_the_discovery_challenge() {
    let h = Harness::start().await;
    let response = h.rpc(None, call("list_projects", None)).await;
    assert_eq!(response.status(), 401);
    assert_eq!(
        response.headers()["www-authenticate"],
        format!("Bearer resource_metadata=\"{METADATA}\"")
    );
    assert!(h.upstream_calls().is_empty());
}

#[tokio::test]
async fn the_metadata_document_names_the_resource_and_the_realm() {
    let h = Harness::start().await;
    for path in [
        "/.well-known/oauth-protected-resource",
        "/.well-known/oauth-protected-resource/mcp",
    ] {
        let doc: Value = h
            .http
            .get(format!("{}{path}", h.gateway))
            .send()
            .await
            .unwrap()
            .json()
            .await
            .unwrap();
        assert_eq!(doc["resource"], RESOURCE);
        assert_eq!(doc["authorization_servers"], json!([h.issuer()]));
        assert_eq!(
            doc["scopes_supported"],
            json!(["mestier.read", "mestier.doe.write"])
        );
        assert_eq!(doc["bearer_methods_supported"], json!(["header"]));
    }
}

#[tokio::test]
async fn tokens_that_are_not_for_this_gateway_are_refused() {
    let h = Harness::start().await;
    let bad = [
        h.token(json!({"aud": "mestier-api"})),
        h.token(json!({"aud": ["other", "mestier-api"]})),
        h.token(json!({"iss": "http://evil.example/realms/mestier"})),
        h.token(json!({"exp": now() - 600})),
        "not-a-jwt".to_owned(),
    ];
    for token in bad {
        let response = h.rpc(Some(&token), call("list_projects", None)).await;
        assert_eq!(response.status(), 401);
        let challenge = response.headers()["www-authenticate"].to_str().unwrap();
        assert!(challenge.contains("error=\"invalid_token\""), "{challenge}");
        assert!(challenge.contains(&format!("resource_metadata=\"{METADATA}\"")));
    }
    assert!(h.upstream_calls().is_empty());
    assert!(h.pdp.requests.lock().unwrap().is_empty());
}

#[tokio::test]
async fn an_audience_list_containing_the_gateway_is_accepted() {
    let h = Harness::start().await;
    let token = h.token(json!({"aud": ["other", RESOURCE]}));
    let response = h.rpc(Some(&token), call("list_projects", None)).await;
    assert_eq!(response.status(), 200);
}

#[tokio::test]
async fn an_unknown_tool_is_refused_before_the_policy() {
    let h = Harness::start().await;
    let token = h.token(json!({}));
    let response = h.rpc(Some(&token), call("delete_project", None)).await;
    let body: Value = response.json().await.unwrap();
    assert_eq!(body["error"]["code"], -32002);
    assert_eq!(body["error"]["message"], "unknown tool");
    assert_eq!(body["id"], 9);
    assert!(h.pdp.requests.lock().unwrap().is_empty());
    assert!(h.upstream_calls().is_empty());
}

#[tokio::test]
async fn a_policy_denial_stops_the_call() {
    let h = Harness::start().await;
    h.set_pdp(PdpMode::Deny);
    let token = h.token(json!({}));
    let response = h.rpc(Some(&token), call("create_doe", Some("acme"))).await;
    let body: Value = response.json().await.unwrap();
    assert_eq!(body["error"]["code"], -32001);
    assert_eq!(body["error"]["message"], "denied by policy");
    assert!(h.upstream_calls().is_empty());
    assert_eq!(h.auth.exchanges.load(Ordering::SeqCst), 0);
}

#[tokio::test]
async fn a_failing_policy_point_means_refusal() {
    let h = Harness::start().await;
    h.set_pdp(PdpMode::Broken);
    let token = h.token(json!({}));
    let body: Value = h
        .rpc(Some(&token), call("list_projects", None))
        .await
        .json()
        .await
        .unwrap();
    assert_eq!(body["error"]["code"], -32001);
    assert!(h.upstream_calls().is_empty());
}

#[tokio::test]
async fn an_unreachable_policy_point_means_refusal() {
    let dead = {
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        format!("http://{}", listener.local_addr().unwrap())
    };
    let h = Harness::start_with(Some(dead)).await;
    let token = h.token(json!({}));
    let body: Value = h
        .rpc(Some(&token), call("list_projects", None))
        .await
        .json()
        .await
        .unwrap();
    assert_eq!(body["error"]["code"], -32001);
    assert!(h.upstream_calls().is_empty());
}

#[tokio::test]
async fn an_allowed_call_reaches_the_server_with_the_delegated_token_only() {
    let h = Harness::start().await;
    let token = h.token(json!({}));
    let response = h.rpc(Some(&token), call("get_doe", Some("acme"))).await;
    assert_eq!(response.status(), 200);
    let body: Value = response.json().await.unwrap();
    assert_eq!(body["result"]["ok"], true);

    let calls = h.upstream_calls();
    assert_eq!(calls.len(), 1);
    assert_eq!(
        calls[0].authorization.as_deref(),
        Some("Bearer delegated-1")
    );
    assert_ne!(
        calls[0].authorization.as_deref(),
        Some(format!("Bearer {token}").as_str())
    );
    assert_eq!(calls[0].client.as_deref(), Some("chatgpt"));

    let requests = h.pdp.requests.lock().unwrap().clone();
    assert_eq!(
        requests,
        vec![json!({
            "subject": {"type": "user", "id": "alice", "properties": {
                "roles": ["user"], "client_id": "chatgpt", "scopes": ["openid", "mestier.read"]}},
            "action": {"name": "get_doe"},
            "resource": {"type": "mcp_tool", "id": "get_doe", "properties": {
                "access": "read", "domain": "doe", "organization": "acme"}},
            "context": {},
        })]
    );

    let forms = h.auth.forms.lock().unwrap().clone();
    let exchange = forms
        .iter()
        .find(|f| f["grant_type"].contains("token-exchange"))
        .unwrap();
    assert_eq!(exchange["subject_token"], token);
    assert_eq!(exchange["audience"], "mestier-api");
    assert_eq!(exchange["actor_token"], "actor-token");
    assert_eq!(exchange["client_id"], "mestier-mcp-gateway");
}

#[tokio::test]
async fn a_call_without_organization_sends_null() {
    let h = Harness::start().await;
    let token = h.token(json!({}));
    h.rpc(Some(&token), call("list_organizations", None)).await;
    let requests = h.pdp.requests.lock().unwrap().clone();
    assert_eq!(
        requests[0]["resource"]["properties"]["organization"],
        Value::Null
    );
}

#[tokio::test]
async fn methods_other_than_tool_calls_skip_the_policy() {
    let h = Harness::start().await;
    let token = h.token(json!({}));
    let response = h
        .rpc(
            Some(&token),
            json!({"jsonrpc": "2.0", "id": 1, "method": "tools/list"}),
        )
        .await;
    assert_eq!(response.status(), 200);
    assert!(h.pdp.requests.lock().unwrap().is_empty());
    assert_eq!(h.upstream_calls().len(), 1);
}

#[tokio::test]
async fn the_delegated_token_is_cached_per_user_and_client() {
    let h = Harness::start().await;
    let alice = h.token(json!({}));
    let bob = h.token(json!({"sub": "bob"}));
    let alice_other_client = h.token(json!({"azp": "claude"}));

    for token in [&alice, &alice, &bob, &alice_other_client] {
        h.rpc(Some(token), call("list_projects", None)).await;
    }

    let seen: Vec<_> = h
        .upstream_calls()
        .into_iter()
        .map(|c| c.authorization.unwrap())
        .collect();
    assert_eq!(
        seen,
        [
            "Bearer delegated-1",
            "Bearer delegated-1",
            "Bearer delegated-2",
            "Bearer delegated-3"
        ]
    );
    assert_eq!(h.auth.exchanges.load(Ordering::SeqCst), 3);
}

#[tokio::test]
async fn a_failed_exchange_is_a_generic_bad_gateway() {
    let h = Harness::start().await;
    h.auth.fail_exchange.store(true, Ordering::SeqCst);
    let token = h.token(json!({}));
    let response = h.rpc(Some(&token), call("list_projects", None)).await;
    assert_eq!(response.status(), 502);
    let text = response.text().await.unwrap();
    assert!(!text.contains("actor-token") && !text.contains(&token));
    assert!(h.upstream_calls().is_empty());
}

#[tokio::test]
async fn session_headers_are_forwarded_and_delete_is_proxied() {
    let h = Harness::start().await;
    let token = h.token(json!({}));
    let response = h
        .http
        .delete(format!("{}/mcp", h.gateway))
        .bearer_auth(&token)
        .header("mcp-session-id", "abc")
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), 200);
    let calls = h.upstream_calls();
    assert_eq!(calls[0].method, "DELETE");
    assert_eq!(calls[0].session.as_deref(), Some("abc"));
    assert_eq!(
        calls[0].authorization.as_deref(),
        Some("Bearer delegated-1")
    );
}

#[tokio::test]
async fn server_sent_events_are_streamed_not_buffered() {
    let h = Harness::start().await;
    let token = h.token(json!({}));
    let mut response = h
        .http
        .get(format!("{}/mcp", h.gateway))
        .bearer_auth(&token)
        .header("accept", "text/event-stream")
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), 200);
    assert_eq!(response.headers()["content-type"], "text/event-stream");

    let first = tokio::time::timeout(Duration::from_secs(5), response.chunk())
        .await
        .expect("first event must arrive before the stream ends")
        .unwrap()
        .unwrap();
    assert!(String::from_utf8_lossy(&first).contains("first"));

    h.upstream.release.notify_one();
    let mut rest = String::new();
    while let Some(chunk) = response.chunk().await.unwrap() {
        rest.push_str(&String::from_utf8_lossy(&chunk));
    }
    assert!(rest.contains("second"));
}
