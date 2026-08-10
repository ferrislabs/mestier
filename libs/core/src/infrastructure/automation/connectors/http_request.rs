//! `http.request`: an outbound HTTP call to wherever the resolved `url`
//! points, authenticated with an `AnyOf(bearer_token, http_basic,
//! http_header)` credential and optionally signed with a second, distinct
//! one. See `domain::automation::connector::catalogue` for the descriptor
//! and `infrastructure::automation::connectors::http_client` for the guard,
//! auth and response-shaping this reuses.

use std::time::Duration;

use chrono::Utc;
use reqwest::{Client, Method, Url};
use serde_json::{Map, Value, json};
use uuid::Uuid;

use crate::application::MestierUseCase;
use crate::domain::automation::run::{Connector, ConnectorInput, ConnectorOutcome};
use crate::infrastructure::automation::webhook::{
    address_policy::PrivateNetworkAccess, signature::sign,
};

use super::http_client::{
    CapturedResponse, DEFAULT_TIMEOUT, auth_from_credential, build_guarded_client,
    describe_reqwest_error, read_response, refuse_literal_private_address,
};

pub struct HttpRequestConnector {
    usecase: MestierUseCase,
    client: Client,
    /// `None` only via the test seam [`Self::with_client`], which bypasses
    /// every network guard on purpose. A production connector always
    /// carries one, and [`Self::execute`] uses it to catch what the guarded
    /// resolver structurally cannot: a literal IP in `url`, which `reqwest`
    /// never hands to a custom `Resolve` because there is nothing to
    /// resolve.
    access: Option<PrivateNetworkAccess>,
}

impl HttpRequestConnector {
    /// The production constructor. It builds the guarded client itself so no
    /// caller can accidentally assemble a connector that skips the guard.
    pub fn new(usecase: MestierUseCase, access: PrivateNetworkAccess) -> Self {
        let client = build_guarded_client(access)
            // The only way `ClientBuilder::build` fails here is a host TLS
            // backend the process cannot recover from — the same class of
            // startup failure `SecretCipher::from_base64` propagates, except
            // this constructor runs deep inside worker start-up rather than
            // `create_service`, so there is no `Result` to propagate through.
            .expect("building the guarded HTTP client failed");
        Self {
            usecase,
            client,
            access: Some(access),
        }
    }

    /// Test seam: lets a test point at a local server, which the guard would
    /// otherwise refuse — loopback is refused unconditionally, on purpose.
    /// Never use it to build a production connector.
    #[cfg(test)]
    fn with_client(usecase: MestierUseCase, client: Client) -> Self {
        Self {
            usecase,
            client,
            access: None,
        }
    }
}

impl Connector for HttpRequestConnector {
    async fn execute(&self, input: ConnectorInput<'_>) -> ConnectorOutcome {
        let method = match read_method(input.config) {
            Ok(method) => method,
            Err(error) => return ConnectorOutcome::Failed { error },
        };
        let Some(url) = input.config.get("url").and_then(Value::as_str) else {
            return ConnectorOutcome::Failed {
                error: "`url` must resolve to a string".to_owned(),
            };
        };
        if let Some(access) = self.access {
            let parsed = match Url::parse(url) {
                Ok(parsed) => parsed,
                Err(error) => {
                    return ConnectorOutcome::Failed {
                        error: format!("`url` is not a valid URL: {error}"),
                    };
                }
            };
            if let Err(error) = refuse_literal_private_address(&parsed, access) {
                return ConnectorOutcome::Failed { error };
            }
        }
        let timeout = input
            .config
            .get("timeout_seconds")
            .and_then(Value::as_u64)
            .map(Duration::from_secs)
            .unwrap_or(DEFAULT_TIMEOUT);

        let headers = match read_headers(input.config) {
            Ok(headers) => headers,
            Err(error) => return ConnectorOutcome::Failed { error },
        };
        let body = input.config.get("body").filter(|v| !v.is_null());

        let mut request = self.client.request(method, url).timeout(timeout);
        let has_content_type = headers
            .iter()
            .any(|(name, _)| name.eq_ignore_ascii_case("content-type"));
        for (name, value) in &headers {
            request = request.header(name, value);
        }

        if let Some(credential_id) = input.credential_id {
            match self.open_auth(input.org_id, credential_id).await {
                Ok(auth) => request = auth.apply(request),
                Err(error) => return ConnectorOutcome::Failed { error },
            }
        }

        let body_bytes = body.map(|value| {
            serde_json::to_vec(value)
                .expect("a serde_json::Value serializes into a JSON body without failing")
        });

        if let Some(signing_id) = input
            .config
            .get("signing_credential_id")
            .and_then(Value::as_str)
        {
            match self
                .sign_headers(
                    input.org_id,
                    signing_id,
                    body_bytes.as_deref().unwrap_or(b""),
                )
                .await
            {
                Ok((timestamp, signature)) => {
                    request = request
                        .header("x-mestier-timestamp", timestamp.to_string())
                        .header("x-mestier-signature", signature);
                }
                Err(error) => return ConnectorOutcome::Failed { error },
            }
        }

        if let Some(bytes) = body_bytes {
            if !has_content_type {
                request = request.header("content-type", "application/json");
            }
            request = request.body(bytes);
        }

        let response = match request.send().await {
            Ok(response) => response,
            Err(error) => {
                return ConnectorOutcome::Failed {
                    error: describe_reqwest_error(&error),
                };
            }
        };

        match read_response(response).await {
            Ok(captured) => outcome_for(captured),
            Err(error) => ConnectorOutcome::Failed { error },
        }
    }
}

impl HttpRequestConnector {
    async fn open_auth(
        &self,
        org_id: common::OrganizationId,
        credential_id: Uuid,
    ) -> Result<super::http_client::HttpAuth, String> {
        let (credential, plaintext) = self
            .usecase
            .open_credential(org_id, credential_id)
            .await
            .map_err(|error| format!("the auth credential is unusable: {error}"))?;

        let fields = parse_credential_fields(&plaintext)
            .ok_or_else(|| "the auth credential's data is not usable".to_owned())?;

        auth_from_credential(&credential.kind, &fields).map_err(|error| error.to_string())
    }

    async fn sign_headers(
        &self,
        org_id: common::OrganizationId,
        signing_credential_id: &str,
        body: &[u8],
    ) -> Result<(i64, String), String> {
        let signing_id = Uuid::parse_str(signing_credential_id)
            .map_err(|_| "`signing_credential_id` must be a UUID".to_owned())?;

        let (_, secret) = self
            .usecase
            .open_credential(org_id, signing_id)
            .await
            .map_err(|error| format!("the signing credential is unusable: {error}"))?;

        let timestamp = Utc::now().timestamp();
        let signature = sign(&secret, timestamp, body);
        Ok((timestamp, signature))
    }
}

fn parse_credential_fields(plaintext: &[u8]) -> Option<Map<String, Value>> {
    match serde_json::from_slice(plaintext).ok()? {
        Value::Object(map) => Some(map),
        _ => None,
    }
}

fn read_method(config: &Map<String, Value>) -> Result<Method, String> {
    let raw = config
        .get("method")
        .and_then(Value::as_str)
        .ok_or_else(|| "`method` is required".to_owned())?;

    Method::from_bytes(raw.as_bytes())
        .map_err(|_| format!("`method` `{raw}` is not a valid HTTP method"))
}

fn read_headers(config: &Map<String, Value>) -> Result<Vec<(String, String)>, String> {
    let Some(headers) = config.get("headers") else {
        return Ok(Vec::new());
    };
    let Some(headers) = headers.as_object() else {
        return Err("`headers` must resolve to an object".to_owned());
    };

    headers
        .iter()
        .map(|(name, value)| {
            value
                .as_str()
                .map(|value| (name.clone(), value.to_owned()))
                .ok_or_else(|| format!("header `{name}` must resolve to a string"))
        })
        .collect()
}

/// Only `2xx` is a success — a redirect (never followed, `Policy::none()`)
/// and every other status go back into the retry schedule, the body kept in
/// the message so the run inspector shows *why* rather than just the code.
fn outcome_for(captured: CapturedResponse) -> ConnectorOutcome {
    if (200..300).contains(&captured.status) {
        ConnectorOutcome::Produced(json!({
            "status": captured.status,
            "headers": captured.headers,
            "body": captured.body,
        }))
    } else {
        ConnectorOutcome::Failed {
            error: format!(
                "the endpoint answered {} with body: {}",
                captured.status, captured.body
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::time::Instant;

    use common::OrganizationId;
    use sqlx::PgPool;

    use super::*;
    use crate::application::default_authorizer;
    use crate::infrastructure::automation::connectors::test_support::{StubResponse, TestServer};
    use crate::infrastructure::realtime::EventHub;

    fn config(pairs: &[(&str, Value)]) -> Map<String, Value> {
        pairs
            .iter()
            .cloned()
            .map(|(k, v)| (k.to_owned(), v))
            .collect()
    }

    /// `connect_lazy` builds a pool without touching the network — enough for
    /// tests that never open a credential.
    fn lazy_usecase() -> MestierUseCase {
        let pool = PgPool::connect_lazy("postgres://unused:unused@localhost/unused")
            .expect("a lazy pool needs no server");
        MestierUseCase::new(pool, default_authorizer(), EventHub::new())
    }

    fn connector_for(client: Client) -> HttpRequestConnector {
        HttpRequestConnector::with_client(lazy_usecase(), client)
    }

    /// A client reaching `127.0.0.1` (which the guard refuses
    /// unconditionally) but otherwise configured exactly like production:
    /// no redirects followed. `Client::new()` alone would follow up to 10 —
    /// its default policy, not `http.request`'s.
    fn test_client() -> Client {
        Client::builder()
            .redirect(reqwest::redirect::Policy::none())
            .build()
            .expect("a plain client builds without a custom resolver")
    }

    fn input(config: &Map<String, Value>) -> ConnectorInput<'_> {
        ConnectorInput {
            org_id: OrganizationId(Uuid::from_u128(1)),
            run_id: Uuid::from_u128(2),
            config,
            credential_id: None,
        }
    }

    #[tokio::test]
    async fn a_missing_method_fails_without_a_network_call() {
        let cfg = config(&[("url", json!("http://example.invalid/"))]);

        let outcome = connector_for(test_client()).execute(input(&cfg)).await;

        assert!(
            matches!(outcome, ConnectorOutcome::Failed { .. }),
            "{outcome:?}"
        );
    }

    #[tokio::test]
    async fn a_missing_url_fails_without_a_network_call() {
        let cfg = config(&[("method", json!("GET"))]);

        let outcome = connector_for(test_client()).execute(input(&cfg)).await;

        assert!(
            matches!(outcome, ConnectorOutcome::Failed { .. }),
            "{outcome:?}"
        );
    }

    #[tokio::test]
    async fn an_invalid_method_fails_without_a_network_call() {
        let cfg = config(&[
            ("method", json!("not-a-method with spaces")),
            ("url", json!("http://example.invalid/")),
        ]);

        let outcome = connector_for(test_client()).execute(input(&cfg)).await;

        assert!(
            matches!(outcome, ConnectorOutcome::Failed { .. }),
            "{outcome:?}"
        );
    }

    /// The end-to-end contract #202 exists for: a call goes out, and the
    /// status, headers and parsed JSON body come back on `Produced`.
    #[tokio::test]
    async fn a_successful_call_exposes_status_headers_and_a_parsed_body() {
        let server = TestServer::respond_always(StubResponse::json(200, &json!({"id": 42}))).await;
        let cfg = config(&[
            ("method", json!("POST")),
            ("url", json!(format!("{}/customers", server.url))),
            ("body", json!({"name": "Ada"})),
        ]);

        let outcome = connector_for(test_client()).execute(input(&cfg)).await;

        let ConnectorOutcome::Produced(value) = outcome else {
            panic!("expected Produced, got {outcome:?}");
        };
        assert_eq!(value["status"], 200);
        assert_eq!(value["body"], json!({"id": 42}));
        assert_eq!(value["headers"]["content-type"], json!("application/json"));

        let request = server.last_request();
        assert_eq!(request.method, "POST");
        assert_eq!(request.path, "/customers");
        assert_eq!(request.body_json(), json!({"name": "Ada"}));
        assert_eq!(
            request.headers.get("content-type").map(String::as_str),
            Some("application/json")
        );
    }

    #[tokio::test]
    async fn a_non_2xx_response_fails_with_the_body_preserved() {
        let server =
            TestServer::respond_always(StubResponse::json(422, &json!({"error": "invalid email"})))
                .await;
        let cfg = config(&[("method", json!("POST")), ("url", json!(server.url))]);

        let outcome = connector_for(test_client()).execute(input(&cfg)).await;

        let ConnectorOutcome::Failed { error } = outcome else {
            panic!("expected Failed, got {outcome:?}");
        };
        assert!(error.contains("422"), "{error}");
        assert!(error.contains("invalid email"), "{error}");
    }

    /// The client is built with `Policy::none()`; a 3xx must surface as an
    /// ordinary failure carrying its status, never as a followed hop.
    #[tokio::test]
    async fn a_redirect_is_a_failure_not_a_followed_hop() {
        // The `location` value is a relative path, not an absolute URL:
        // `reqwest` inspects it even under `Policy::none()`, and an absolute
        // target needing DNS resolution (an RFC 2606 `.invalid` name, in
        // particular) turns "do not follow" into a connection error instead
        // of the plain 3xx response this test wants to assert on.
        let server = TestServer::respond_always(
            StubResponse::text(302, "").with_header("location", "/elsewhere"),
        )
        .await;
        let cfg = config(&[("method", json!("GET")), ("url", json!(server.url))]);

        let outcome = connector_for(test_client()).execute(input(&cfg)).await;

        let ConnectorOutcome::Failed { error } = outcome else {
            panic!("expected Failed, got {outcome:?}");
        };
        assert!(error.contains("302"), "{error}");
        assert_eq!(
            server.requests().len(),
            1,
            "the redirect target must never be reached"
        );
    }

    /// Bounded wall time is the proof: a blocking implementation would hang
    /// well past the configured timeout, and the run engine's worker with it.
    #[tokio::test]
    async fn a_slow_endpoint_times_out_without_blocking() {
        let server = TestServer::start(|_| {
            std::thread::sleep(Duration::from_secs(5));
            StubResponse::text(200, "too late")
        })
        .await;
        let cfg = config(&[
            ("method", json!("GET")),
            ("url", json!(server.url)),
            ("timeout_seconds", json!(1)),
        ]);

        let started = Instant::now();
        let outcome = connector_for(test_client()).execute(input(&cfg)).await;

        assert!(
            matches!(outcome, ConnectorOutcome::Failed { .. }),
            "{outcome:?}"
        );
        assert!(
            started.elapsed() < Duration::from_secs(4),
            "the connector must fail close to the configured timeout, took {:?}",
            started.elapsed()
        );
    }

    /// Success and truncation are independent: an oversized body is still a
    /// `Produced` outcome, just with its body capped and marked.
    #[tokio::test]
    async fn an_oversized_response_is_truncated_with_its_marker() {
        let big = "a".repeat(super::super::http_client::MAX_RESPONSE_BODY_BYTES + 1000);
        let server = TestServer::respond_always(StubResponse::text(200, big)).await;
        let cfg = config(&[("method", json!("GET")), ("url", json!(server.url))]);

        let outcome = connector_for(test_client()).execute(input(&cfg)).await;

        let ConnectorOutcome::Produced(value) = outcome else {
            panic!("expected Produced, got {outcome:?}");
        };
        let body = value["body"]
            .as_str()
            .expect("a truncated body is a string");
        assert!(
            body.ends_with(super::super::http_client::TRUNCATION_MARKER),
            "{body}"
        );
        assert!(body.len() <= super::super::http_client::MAX_RESPONSE_BODY_BYTES + 100);
    }

    /// `10.255.255.1` is never routed to from this test process; the point
    /// is not that the call succeeds but *where* it fails: at the resolver,
    /// naming the refusal, when denied — past it, on the network, when
    /// allowed. Loopback cannot stand in for this case: the guard refuses it
    /// unconditionally regardless of policy.
    #[tokio::test]
    async fn a_private_target_is_refused_by_the_resolver_when_denied() {
        let cfg = config(&[
            ("method", json!("GET")),
            ("url", json!("http://10.255.255.1:9/probe")),
            ("timeout_seconds", json!(1)),
        ]);
        let connector = HttpRequestConnector::new(lazy_usecase(), PrivateNetworkAccess::Denied);

        let outcome = connector.execute(input(&cfg)).await;

        let ConnectorOutcome::Failed { error } = outcome else {
            panic!("expected Failed, got {outcome:?}");
        };
        assert!(error.contains("private address"), "{error}");
    }

    #[tokio::test]
    async fn a_private_target_reaches_the_network_when_allowed() {
        let cfg = config(&[
            ("method", json!("GET")),
            ("url", json!("http://10.255.255.1:9/probe")),
            ("timeout_seconds", json!(1)),
        ]);
        let connector = HttpRequestConnector::new(lazy_usecase(), PrivateNetworkAccess::Allowed);

        let outcome = connector.execute(input(&cfg)).await;

        let ConnectorOutcome::Failed { error } = outcome else {
            panic!("expected Failed, got {outcome:?}");
        };
        assert!(
            !error.contains("private address"),
            "the resolver must not have refused it: {error}"
        );
    }

    #[test]
    fn only_2xx_is_a_success() {
        let produced = outcome_for(CapturedResponse {
            status: 204,
            headers: json!({}),
            body: Value::Null,
        });
        assert!(matches!(produced, ConnectorOutcome::Produced(_)));

        let failed = outcome_for(CapturedResponse {
            status: 500,
            headers: json!({}),
            body: Value::Null,
        });
        assert!(matches!(failed, ConnectorOutcome::Failed { .. }));
    }

    // --- Auth and signing: these open a real credential, so they need a
    // live database, following the same convention as
    // `application::automation::credential`'s own tests.

    async fn make_pool() -> PgPool {
        let url = std::env::var("DATABASE_URL")
            .expect("DATABASE_URL must be set to run http.request auth integration tests");
        PgPool::connect(&url).await.unwrap()
    }

    async fn seed_organization(pool: &PgPool, label: &str) -> OrganizationId {
        let owner_id = common::generate_uuid_v7();
        sqlx::query!(
            r#"INSERT INTO users (id, email, username, display_name, sub)
               VALUES ($1, $2, $3, $4, $5)"#,
            owner_id,
            format!("owner-{owner_id}@example.com"),
            format!("owner-{owner_id}"),
            "Owner User",
            format!("sub-owner-{owner_id}"),
        )
        .execute(pool)
        .await
        .unwrap();

        let org_id = common::generate_uuid_v7();
        sqlx::query!(
            r#"INSERT INTO organizations (id, name, slug, owner_id)
               VALUES ($1, $2, $3, $4)"#,
            org_id,
            format!("{label} Org"),
            format!("{label}-{org_id}"),
            owner_id,
        )
        .execute(pool)
        .await
        .unwrap();

        OrganizationId(org_id)
    }

    fn usecase_with_cipher(pool: PgPool) -> MestierUseCase {
        use crate::infrastructure::automation::webhook::secret::SecretCipher;
        use std::sync::Arc;

        let mut usecase = MestierUseCase::new(pool, default_authorizer(), EventHub::new());
        usecase.cipher = Some(Arc::new(SecretCipher::new(&[7u8; 32]).unwrap()));
        usecase
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn a_bearer_token_credential_produces_an_authorization_bearer_header() {
        let pool = make_pool().await;
        let org_id = seed_organization(&pool, "auth-bearer").await;
        let usecase = usecase_with_cipher(pool);
        let credential = usecase
            .create_credential(
                crate::domain::automation::credential::CreateCredentialCommand {
                    org_id,
                    kind: "bearer_token".to_owned(),
                    name: "API token".to_owned(),
                    origin: crate::domain::automation::credential::CredentialOrigin::Supplied,
                    data: Some(json!({ "token": "abc123" })),
                },
            )
            .await
            .unwrap();
        let server = TestServer::respond_always(StubResponse::text(200, "ok")).await;
        let cfg = config(&[("method", json!("GET")), ("url", json!(server.url))]);
        let connector = HttpRequestConnector::with_client(usecase, test_client());

        let outcome = connector
            .execute(ConnectorInput {
                org_id,
                run_id: Uuid::from_u128(2),
                config: &cfg,
                credential_id: Some(credential.id),
            })
            .await;

        assert!(
            matches!(outcome, ConnectorOutcome::Produced(_)),
            "{outcome:?}"
        );
        let request = server.last_request();
        assert_eq!(
            request.headers.get("authorization").map(String::as_str),
            Some("Bearer abc123")
        );
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn an_http_basic_credential_produces_a_base64_authorization_header() {
        let pool = make_pool().await;
        let org_id = seed_organization(&pool, "auth-basic").await;
        let usecase = usecase_with_cipher(pool);
        let credential = usecase
            .create_credential(
                crate::domain::automation::credential::CreateCredentialCommand {
                    org_id,
                    kind: "http_basic".to_owned(),
                    name: "Basic auth".to_owned(),
                    origin: crate::domain::automation::credential::CredentialOrigin::Supplied,
                    data: Some(json!({ "username": "bot", "password": "secret" })),
                },
            )
            .await
            .unwrap();
        let server = TestServer::respond_always(StubResponse::text(200, "ok")).await;
        let cfg = config(&[("method", json!("GET")), ("url", json!(server.url))]);
        let connector = HttpRequestConnector::with_client(usecase, test_client());

        let outcome = connector
            .execute(ConnectorInput {
                org_id,
                run_id: Uuid::from_u128(2),
                config: &cfg,
                credential_id: Some(credential.id),
            })
            .await;

        assert!(
            matches!(outcome, ConnectorOutcome::Produced(_)),
            "{outcome:?}"
        );
        let request = server.last_request();
        use base64::{Engine, engine::general_purpose::STANDARD};
        let expected = format!("Basic {}", STANDARD.encode("bot:secret"));
        assert_eq!(
            request.headers.get("authorization").map(String::as_str),
            Some(expected.as_str())
        );
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn an_http_header_credential_produces_the_named_custom_header() {
        let pool = make_pool().await;
        let org_id = seed_organization(&pool, "auth-header").await;
        let usecase = usecase_with_cipher(pool);
        let credential = usecase
            .create_credential(
                crate::domain::automation::credential::CreateCredentialCommand {
                    org_id,
                    kind: "http_header".to_owned(),
                    name: "Header auth".to_owned(),
                    origin: crate::domain::automation::credential::CredentialOrigin::Supplied,
                    data: Some(json!({ "header_name": "x-api-key", "header_value": "shh" })),
                },
            )
            .await
            .unwrap();
        let server = TestServer::respond_always(StubResponse::text(200, "ok")).await;
        let cfg = config(&[("method", json!("GET")), ("url", json!(server.url))]);
        let connector = HttpRequestConnector::with_client(usecase, test_client());

        let outcome = connector
            .execute(ConnectorInput {
                org_id,
                run_id: Uuid::from_u128(2),
                config: &cfg,
                credential_id: Some(credential.id),
            })
            .await;

        assert!(
            matches!(outcome, ConnectorOutcome::Produced(_)),
            "{outcome:?}"
        );
        let request = server.last_request();
        assert_eq!(
            request.headers.get("x-api-key").map(String::as_str),
            Some("shh")
        );
    }

    /// The signature #202 produces must be exactly what `signature::verify`
    /// accepts — documentation and implementation cannot drift because both
    /// sides live in `signature.rs`; this proves the connector wires into it
    /// correctly.
    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn a_signing_credential_produces_a_signature_verify_accepts() {
        let pool = make_pool().await;
        let org_id = seed_organization(&pool, "signing").await;
        let usecase = usecase_with_cipher(pool);
        let signing = usecase
            .create_credential(
                crate::domain::automation::credential::CreateCredentialCommand {
                    org_id,
                    kind: "bearer_token".to_owned(),
                    name: "Outgoing signature".to_owned(),
                    origin: crate::domain::automation::credential::CredentialOrigin::Generated,
                    data: None,
                },
            )
            .await
            .unwrap();
        let (_, secret) = usecase.open_credential(org_id, signing.id).await.unwrap();
        let server = TestServer::respond_always(StubResponse::text(200, "ok")).await;
        let cfg = config(&[
            ("method", json!("POST")),
            ("url", json!(server.url)),
            ("body", json!({"total_cents": 100})),
            ("signing_credential_id", json!(signing.id.to_string())),
        ]);
        let connector = HttpRequestConnector::with_client(usecase, test_client());

        let outcome = connector
            .execute(ConnectorInput {
                org_id,
                run_id: Uuid::from_u128(2),
                config: &cfg,
                credential_id: None,
            })
            .await;

        assert!(
            matches!(outcome, ConnectorOutcome::Produced(_)),
            "{outcome:?}"
        );
        let request = server.last_request();
        let timestamp: i64 = request
            .headers
            .get("x-mestier-timestamp")
            .expect("timestamp header")
            .parse()
            .expect("a numeric timestamp");
        let signature = request
            .headers
            .get("x-mestier-signature")
            .expect("signature header");

        assert!(
            crate::infrastructure::automation::webhook::signature::verify(
                &secret,
                timestamp,
                &request.body,
                signature,
            ),
            "a receiver holding the secret must be able to verify what arrived"
        );
    }
}
