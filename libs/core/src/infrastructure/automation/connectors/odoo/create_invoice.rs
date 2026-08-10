//! `odoo.create_invoice`: creates an `account.move` customer invoice with a
//! single line. See `domain::automation::connector::catalogue` for the
//! descriptor and `super` for the shared authenticate-then-`execute_kw`
//! plumbing.

use serde_json::{Value, json};

use crate::application::MestierUseCase;
use crate::domain::automation::run::{Connector, ConnectorInput, ConnectorOutcome};
use crate::infrastructure::automation::webhook::address_policy::PrivateNetworkAccess;

use super::OdooClient;

pub struct OdooCreateInvoiceConnector {
    client: OdooClient,
}

impl OdooCreateInvoiceConnector {
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

impl Connector for OdooCreateInvoiceConnector {
    async fn execute(&self, input: ConnectorInput<'_>) -> ConnectorOutcome {
        let Some(partner_id) = input.config.get("partner_id").and_then(Value::as_i64) else {
            return ConnectorOutcome::Failed {
                error: "`partner_id` must resolve to a number".to_owned(),
            };
        };
        let Some(description) = input.config.get("description").and_then(Value::as_str) else {
            return ConnectorOutcome::Failed {
                error: "`description` must resolve to a string".to_owned(),
            };
        };
        let Some(amount) = input.config.get("amount").and_then(Value::as_f64) else {
            return ConnectorOutcome::Failed {
                error: "`amount` must resolve to a number".to_owned(),
            };
        };
        let Some(credential_id) = input.credential_id else {
            return ConnectorOutcome::Failed {
                error: "odoo.create_invoice requires an odoo_api credential".to_owned(),
            };
        };

        let invoice = json!({
            "partner_id": partner_id,
            "move_type": "out_invoice",
            "invoice_line_ids": [[0, 0, {
                "name": description,
                "quantity": 1,
                "price_unit": amount,
            }]],
        });

        match self
            .client
            .call(
                input.org_id,
                credential_id,
                "account.move",
                "create",
                json!([invoice]),
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
    async fn a_missing_amount_fails_without_a_network_call() {
        let cfg = config(&[
            ("partner_id", json!(1)),
            ("description", json!("Consulting")),
        ]);

        let outcome = OdooCreateInvoiceConnector::with_client(lazy_usecase(), test_client())
            .execute(input(&cfg, Some(Uuid::from_u128(9))))
            .await;

        assert!(
            matches!(outcome, ConnectorOutcome::Failed { .. }),
            "{outcome:?}"
        );
    }

    async fn make_pool() -> PgPool {
        let url = std::env::var("DATABASE_URL")
            .expect("DATABASE_URL must be set to run odoo.create_invoice integration tests");
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
        let credential = usecase
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

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn a_successful_call_creates_an_invoice_and_returns_its_id() {
        let pool = make_pool().await;
        let org_id = seed_organization(&pool, "odoo-create-invoice").await;
        let usecase = usecase_with_cipher(pool);
        let server = TestServer::start(|request| {
            let body = request.body_json();
            match body["params"]["method"].as_str().unwrap_or_default() {
                "authenticate" => StubResponse::json(200, &json!({"jsonrpc": "2.0", "result": 2})),
                "execute_kw" => StubResponse::json(200, &json!({"jsonrpc": "2.0", "result": 99})),
                other => panic!("unexpected Odoo method {other}"),
            }
        })
        .await;
        let credential_id = seed_odoo_credential(&usecase, org_id, &server.url).await;
        let cfg = config(&[
            ("partner_id", json!(17)),
            ("description", json!("Consulting — August")),
            ("amount", json!(1500.0)),
        ]);
        let connector = OdooCreateInvoiceConnector::with_client(usecase, test_client());

        let outcome = connector
            .execute(ConnectorInput {
                org_id,
                run_id: Uuid::from_u128(2),
                config: &cfg,
                credential_id: Some(credential_id),
            })
            .await;

        assert_eq!(outcome, ConnectorOutcome::Produced(json!({ "id": 99 })));
        let create_args = server.requests()[1].body_json()["params"]["args"].clone();
        assert_eq!(create_args[3], "account.move");
        assert_eq!(create_args[4], "create");
        assert_eq!(create_args[5][0]["partner_id"], 17);
        assert_eq!(create_args[5][0]["move_type"], "out_invoice");
        assert_eq!(
            create_args[5][0]["invoice_line_ids"][0][2]["price_unit"],
            1500.0
        );
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn an_odoo_side_http_failure_is_a_failure_with_the_status() {
        let pool = make_pool().await;
        let org_id = seed_organization(&pool, "odoo-http-failure").await;
        let usecase = usecase_with_cipher(pool);
        let server = TestServer::respond_always(StubResponse::text(500, "internal error")).await;
        let credential_id = seed_odoo_credential(&usecase, org_id, &server.url).await;
        let cfg = config(&[
            ("partner_id", json!(1)),
            ("description", json!("Consulting")),
            ("amount", json!(100.0)),
        ]);
        let connector = OdooCreateInvoiceConnector::with_client(usecase, test_client());

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
        assert!(error.contains("500"), "{error}");
    }
}
