//! `odoo.create_partner`: creates a `res.partner` record. See
//! `domain::automation::connector::catalogue` for the descriptor and
//! `super` for the shared authenticate-then-`execute_kw` plumbing.

use serde_json::{Value, json};

use crate::application::MestierUseCase;
use crate::domain::automation::run::{Connector, ConnectorInput, ConnectorOutcome};
use crate::infrastructure::automation::webhook::address_policy::PrivateNetworkAccess;

use super::OdooClient;

pub struct OdooCreatePartnerConnector {
    client: OdooClient,
}

impl OdooCreatePartnerConnector {
    pub fn new(usecase: MestierUseCase, access: PrivateNetworkAccess) -> Self {
        Self {
            client: OdooClient::new(usecase, access),
        }
    }

    #[cfg(test)]
    fn with_client(usecase: MestierUseCase, client: reqwest::Client) -> Self {
        Self {
            client: OdooClient::with_client(usecase, client),
        }
    }
}

impl Connector for OdooCreatePartnerConnector {
    async fn execute(&self, input: ConnectorInput<'_>) -> ConnectorOutcome {
        let Some(name) = input.config.get("name").and_then(Value::as_str) else {
            return ConnectorOutcome::Failed {
                error: "`name` must resolve to a string".to_owned(),
            };
        };
        let Some(credential_id) = input.credential_id else {
            return ConnectorOutcome::Failed {
                error: "odoo.create_partner requires an odoo_api credential".to_owned(),
            };
        };

        let mut fields = serde_json::Map::new();
        fields.insert("name".to_owned(), json!(name));
        if let Some(email) = input.config.get("email").and_then(Value::as_str) {
            fields.insert("email".to_owned(), json!(email));
        }
        if let Some(phone) = input.config.get("phone").and_then(Value::as_str) {
            fields.insert("phone".to_owned(), json!(phone));
        }

        match self
            .client
            .call(
                input.org_id,
                credential_id,
                "res.partner",
                "create",
                json!([fields]),
            )
            .await
        {
            Ok(id) => ConnectorOutcome::Produced(json!({ "id": id })),
            Err(error) => ConnectorOutcome::Failed { error },
        }
    }
}

#[cfg(test)]
mod tests {
    use common::OrganizationId;
    use reqwest::Client;
    use sqlx::PgPool;
    use uuid::Uuid;

    use super::*;
    use crate::application::default_authorizer;
    use crate::domain::automation::credential::{CreateCredentialCommand, CredentialOrigin};
    use crate::infrastructure::automation::connectors::test_support::{StubResponse, TestServer};
    use crate::infrastructure::realtime::EventHub;

    fn config(pairs: &[(&str, Value)]) -> serde_json::Map<String, Value> {
        pairs
            .iter()
            .cloned()
            .map(|(k, v)| (k.to_owned(), v))
            .collect()
    }

    fn lazy_usecase() -> MestierUseCase {
        let pool = PgPool::connect_lazy("postgres://unused:unused@localhost/unused")
            .expect("a lazy pool needs no server");
        MestierUseCase::new(pool, default_authorizer(), EventHub::new())
    }

    fn input(
        config: &serde_json::Map<String, Value>,
        credential_id: Option<Uuid>,
    ) -> ConnectorInput<'_> {
        ConnectorInput {
            org_id: OrganizationId(Uuid::from_u128(1)),
            run_id: Uuid::from_u128(2),
            config,
            credential_id,
        }
    }

    fn test_client() -> Client {
        Client::builder()
            .redirect(reqwest::redirect::Policy::none())
            .build()
            .expect("a plain client builds without a custom resolver")
    }

    #[tokio::test]
    async fn a_missing_name_fails_without_a_network_call() {
        let cfg = config(&[]);

        let outcome = OdooCreatePartnerConnector::with_client(lazy_usecase(), test_client())
            .execute(input(&cfg, Some(Uuid::from_u128(9))))
            .await;

        assert!(
            matches!(outcome, ConnectorOutcome::Failed { .. }),
            "{outcome:?}"
        );
    }

    #[tokio::test]
    async fn a_missing_credential_fails_without_a_network_call() {
        let cfg = config(&[("name", json!("Ada"))]);

        let outcome = OdooCreatePartnerConnector::with_client(lazy_usecase(), test_client())
            .execute(input(&cfg, None))
            .await;

        assert!(
            matches!(outcome, ConnectorOutcome::Failed { .. }),
            "{outcome:?}"
        );
    }

    // --- Integration: these open a real credential, so they need Postgres.

    async fn make_pool() -> PgPool {
        let url = std::env::var("DATABASE_URL")
            .expect("DATABASE_URL must be set to run odoo.create_partner integration tests");
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

    async fn seed_odoo_credential(
        usecase: &MestierUseCase,
        org_id: OrganizationId,
        base_url: &str,
    ) -> Uuid {
        let (credential, _secret) = usecase
            .create_credential(CreateCredentialCommand {
                org_id,
                kind: "odoo_api".to_owned(),
                name: "Production Odoo".to_owned(),
                origin: CredentialOrigin::Supplied,
                data: Some(json!({
                    "base_url": base_url,
                    "database": "prod",
                    "username": "bot",
                    "api_key": "correct-key",
                })),
            })
            .await
            .unwrap();
        credential.id
    }

    /// The JSON-RPC handler a happy-path Odoo stub needs: `authenticate`
    /// answers a uid, `execute_kw` answers whatever `create` gave it.
    fn happy_stub(
        created_id: i64,
    ) -> impl Fn(
        &crate::infrastructure::automation::connectors::test_support::CapturedRequest,
    ) -> StubResponse {
        move |request| {
            let body = request.body_json();
            let method = body["params"]["method"].as_str().unwrap_or_default();
            match method {
                "authenticate" => StubResponse::json(200, &json!({"jsonrpc": "2.0", "result": 2})),
                "execute_kw" => {
                    StubResponse::json(200, &json!({"jsonrpc": "2.0", "result": created_id}))
                }
                other => panic!("unexpected Odoo method {other}"),
            }
        }
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn a_successful_call_creates_a_partner_and_returns_its_id() {
        let pool = make_pool().await;
        let org_id = seed_organization(&pool, "odoo-create-partner").await;
        let usecase = usecase_with_cipher(pool);
        let server = TestServer::start(happy_stub(42)).await;
        let credential_id = seed_odoo_credential(&usecase, org_id, &server.url).await;
        let cfg = config(&[
            ("name", json!("Ada Lovelace")),
            ("email", json!("ada@example.com")),
        ]);
        let connector = OdooCreatePartnerConnector::with_client(usecase, test_client());

        let outcome = connector
            .execute(ConnectorInput {
                org_id,
                run_id: Uuid::from_u128(2),
                config: &cfg,
                credential_id: Some(credential_id),
            })
            .await;

        assert_eq!(outcome, ConnectorOutcome::Produced(json!({ "id": 42 })));
        let requests = server.requests();
        assert_eq!(requests.len(), 2, "authenticate, then execute_kw");
        let create_args = requests[1].body_json()["params"]["args"].clone();
        assert_eq!(create_args[3], "res.partner");
        assert_eq!(create_args[4], "create");
        assert_eq!(create_args[5][0]["name"], "Ada Lovelace");
        assert_eq!(create_args[5][0]["email"], "ada@example.com");
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn a_rejected_credential_names_it() {
        let pool = make_pool().await;
        let org_id = seed_organization(&pool, "odoo-bad-credential").await;
        let usecase = usecase_with_cipher(pool);
        let server = TestServer::start(|request| {
            let body = request.body_json();
            let method = body["params"]["method"].as_str().unwrap_or_default();
            match method {
                // Odoo's real behavior for a rejected login: `result: false`.
                "authenticate" => {
                    StubResponse::json(200, &json!({"jsonrpc": "2.0", "result": false}))
                }
                other => panic!("unexpected Odoo method {other}, execute_kw must never run"),
            }
        })
        .await;
        let credential_id = seed_odoo_credential(&usecase, org_id, &server.url).await;
        let cfg = config(&[("name", json!("Ada"))]);
        let connector = OdooCreatePartnerConnector::with_client(usecase, test_client());

        let outcome = connector
            .execute(ConnectorInput {
                org_id,
                run_id: Uuid::from_u128(2),
                config: &cfg,
                credential_id: Some(credential_id),
            })
            .await;

        let ConnectorOutcome::Failed { error } = outcome else {
            panic!("expected Failed, got {outcome:?}");
        };
        assert!(error.contains("Production Odoo"), "{error}");
        assert_eq!(
            server.requests().len(),
            1,
            "execute_kw must never be attempted"
        );
    }
}
