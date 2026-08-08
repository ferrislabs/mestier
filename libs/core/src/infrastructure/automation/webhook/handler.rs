use std::{sync::Arc, time::Duration};

use chrono::Utc;
use common::CoreError;
use reqwest::{Client, StatusCode, redirect::Policy};
use serde_json::json;
use sqlx::PgPool;
use uuid::Uuid;

use super::{
    address_policy::PrivateNetworkAccess, resolver::GuardedResolver, secret::SecretCipher,
    signature::sign,
};
use crate::domain::automation::endpoint::SealedSecret;
use crate::domain::automation::ports::{DeliveryHandler, DeliveryOutcome, DueDelivery};

/// A stranger's endpoint is allowed ten seconds. Beyond that the delivery is a
/// failure and goes back into the retry schedule — a slow receiver must not
/// hold a worker.
const TIMEOUT: Duration = Duration::from_secs(10);

pub struct WebhookDeliveryHandler {
    pool: PgPool,
    /// Shared with the use case that seals secrets: one key, one place.
    cipher: Arc<SecretCipher>,
    client: Client,
}

impl WebhookDeliveryHandler {
    /// The production constructor. It builds the HTTP client itself so no
    /// caller can accidentally assemble one without the guard.
    pub fn new(
        pool: PgPool,
        cipher: Arc<SecretCipher>,
        access: PrivateNetworkAccess,
    ) -> Result<Self, CoreError> {
        let client = Client::builder()
            .timeout(TIMEOUT)
            // Not followed at all. A legitimate webhook has no use for one, and
            // following it would re-open the hole the guarded resolver closes:
            // the redirect target is resolved by the same client, but a 3xx to
            // a fresh name is exactly the shape of an attempted bypass.
            .redirect(Policy::none())
            .dns_resolver(GuardedResolver::new(access))
            .build()
            .map_err(|error| {
                CoreError::Internal(format!("building the webhook client failed: {error}"))
            })?;

        Ok(Self {
            pool,
            cipher,
            client,
        })
    }

    /// Test seam: lets a test point at a local server, which the guard would
    /// otherwise refuse. Never use it to build a production client.
    #[cfg(test)]
    pub fn with_client(pool: PgPool, cipher: Arc<SecretCipher>, client: Client) -> Self {
        Self {
            pool,
            cipher,
            client,
        }
    }

    async fn target(&self, target_id: Uuid) -> Result<(String, Vec<u8>), CoreError> {
        let row = sqlx::query!(
            r#"SELECT url, secret_nonce, secret_ciphertext
               FROM automation.webhook_endpoint
               WHERE id = $1 AND enabled"#,
            target_id,
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(crate::infrastructure::postgres::error::map_sqlx_error)?
        .ok_or(CoreError::NotFound)?;

        let secret = self.cipher.open(&SealedSecret {
            nonce: row.secret_nonce,
            ciphertext: row.secret_ciphertext,
        })?;

        Ok((row.url, secret))
    }
}

/// Only `2xx` is a success. Everything else — including a redirect, which is
/// never followed — goes back into the retry schedule.
fn outcome_for(status: StatusCode) -> DeliveryOutcome {
    if status.is_success() {
        DeliveryOutcome::Succeeded
    } else {
        DeliveryOutcome::Failed {
            error: format!("the endpoint answered {status}"),
        }
    }
}

impl DeliveryHandler for WebhookDeliveryHandler {
    async fn deliver(&self, delivery: &DueDelivery) -> DeliveryOutcome {
        let (url, secret) = match self.target(delivery.target_id).await {
            Ok(target) => target,
            Err(error) => {
                return DeliveryOutcome::Failed {
                    error: format!("the endpoint is unusable: {error}"),
                };
            }
        };

        let body = json!({
            "id": delivery.event.id,
            "name": delivery.event.name,
            "version": delivery.event.version,
            "occurred_at": delivery.event.occurred_at,
            "subject": {
                "kind": delivery.event.subject_kind,
                "id": delivery.event.subject_id,
            },
            "actor": delivery.event.actor,
            "payload": delivery.event.payload,
        })
        .to_string();

        let timestamp = Utc::now().timestamp();
        let signature = sign(&secret, timestamp, body.as_bytes());

        let response = self
            .client
            .post(&url)
            .header("content-type", "application/json")
            // The receiver deduplicates on this: delivery is at-least-once.
            .header("x-mestier-event-id", delivery.event.id.to_string())
            .header("x-mestier-timestamp", timestamp.to_string())
            .header("x-mestier-signature", signature)
            .body(body)
            .send()
            .await;

        match response {
            Ok(response) => outcome_for(response.status()),
            Err(error) => DeliveryOutcome::Failed {
                error: error.to_string(),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use tokio::{
        io::{AsyncReadExt, AsyncWriteExt},
        net::TcpListener,
    };

    use super::*;

    /// Captures one raw HTTP request so the headers and the signed body can be
    /// asserted exactly as a receiver would see them.
    struct Captured {
        headers: HashMap<String, String>,
        body: String,
    }

    async fn serve_once(status_line: &'static str) -> (String, tokio::task::JoinHandle<Captured>) {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();

        let handle = tokio::spawn(async move {
            let (mut socket, _) = listener.accept().await.unwrap();
            let mut buffer = vec![0u8; 8192];
            let read = socket.read(&mut buffer).await.unwrap();
            let request = String::from_utf8_lossy(&buffer[..read]).to_string();

            socket
                .write_all(
                    format!("HTTP/1.1 {status_line}\r\ncontent-length: 0\r\n\r\n").as_bytes(),
                )
                .await
                .unwrap();
            socket.flush().await.unwrap();

            let (head, body) = request.split_once("\r\n\r\n").unwrap_or((&request, ""));
            let headers = head
                .lines()
                .skip(1)
                .filter_map(|line| line.split_once(':'))
                .map(|(name, value)| (name.trim().to_lowercase(), value.trim().to_owned()))
                .collect();

            Captured {
                headers,
                body: body.to_owned(),
            }
        });

        (format!("http://{address}/hook"), handle)
    }

    async fn make_pool() -> PgPool {
        let url = std::env::var("DATABASE_URL")
            .expect("DATABASE_URL must be set to run webhook integration tests");
        PgPool::connect(&url).await.unwrap()
    }

    async fn seed_endpoint(pool: &PgPool, url: &str, cipher: &SecretCipher) -> (Uuid, Vec<u8>) {
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
            "Test Org",
            format!("test-org-{org_id}"),
            owner_id,
        )
        .execute(pool)
        .await
        .unwrap();

        let secret = cipher.generate_secret().unwrap();
        let sealed = cipher.seal(&secret).unwrap();
        let endpoint_id = common::generate_uuid_v7();
        sqlx::query!(
            r#"INSERT INTO automation.webhook_endpoint
                   (id, org_id, url, secret_nonce, secret_ciphertext)
               VALUES ($1, $2, $3, $4, $5)"#,
            endpoint_id,
            org_id,
            url,
            sealed.nonce,
            sealed.ciphertext,
        )
        .execute(pool)
        .await
        .unwrap();

        (endpoint_id, secret)
    }

    fn due(target_id: Uuid, org_id: common::OrganizationId) -> DueDelivery {
        use events::{Actor, EventEnvelope};

        DueDelivery {
            id: common::generate_uuid_v7(),
            org_id,
            subscription_id: common::generate_uuid_v7(),
            kind: "webhook".to_owned(),
            target_id,
            attempts: 0,
            event: EventEnvelope {
                id: Uuid::from_u128(42),
                org_id,
                name: "quote.accepted".to_owned(),
                version: 1,
                subject_kind: "quote".to_owned(),
                subject_id: Some(Uuid::from_u128(7)),
                payload: json!({ "total_cents": 550_000 }),
                actor: Actor::system(),
                correlation_id: None,
                occurred_at: Utc::now(),
            },
        }
    }

    /// The end-to-end contract, asserted the way a receiver would check it:
    /// the headers arrive, and the signature verifies over the body that was
    /// actually sent with the timestamp that was actually sent.
    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn a_receiver_sees_headers_and_a_signature_that_verifies() {
        let pool = make_pool().await;
        let cipher = SecretCipher::new(&[3u8; 32]).unwrap();
        let (url, server) = serve_once("200 OK").await;
        let (endpoint_id, secret) = seed_endpoint(&pool, &url, &cipher).await;
        let org_id = common::OrganizationId(common::generate_uuid_v7());
        // A plain client: the guard would refuse 127.0.0.1, and it has its own
        // tests. What is under test here is the request we build.
        let handler = WebhookDeliveryHandler::with_client(pool, Arc::new(cipher), Client::new());

        let outcome = handler.deliver(&due(endpoint_id, org_id)).await;

        assert_eq!(outcome, DeliveryOutcome::Succeeded);
        let captured = server.await.unwrap();
        assert_eq!(
            captured
                .headers
                .get("x-mestier-event-id")
                .map(String::as_str),
            Some(Uuid::from_u128(42).to_string().as_str())
        );
        let timestamp: i64 = captured
            .headers
            .get("x-mestier-timestamp")
            .expect("timestamp header")
            .parse()
            .expect("a numeric timestamp");
        let signature = captured
            .headers
            .get("x-mestier-signature")
            .expect("signature header");

        assert!(
            super::super::signature::verify(
                &secret,
                timestamp,
                captured.body.as_bytes(),
                signature
            ),
            "a receiver holding the secret must be able to verify what arrived"
        );
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn an_endpoint_that_refuses_is_a_failure_not_a_crash() {
        let pool = make_pool().await;
        let cipher = SecretCipher::new(&[3u8; 32]).unwrap();
        let (url, server) = serve_once("500 Internal Server Error").await;
        let (endpoint_id, _) = seed_endpoint(&pool, &url, &cipher).await;
        let org_id = common::OrganizationId(common::generate_uuid_v7());
        let handler = WebhookDeliveryHandler::with_client(pool, Arc::new(cipher), Client::new());

        let outcome = handler.deliver(&due(endpoint_id, org_id)).await;

        assert!(matches!(outcome, DeliveryOutcome::Failed { .. }));
        let _ = server.await;
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn a_disabled_endpoint_is_a_failure_rather_than_a_silent_success() {
        let pool = make_pool().await;
        let cipher = SecretCipher::new(&[3u8; 32]).unwrap();
        let (endpoint_id, _) = seed_endpoint(&pool, "http://example.invalid/hook", &cipher).await;
        sqlx::query!(
            "UPDATE automation.webhook_endpoint SET enabled = false WHERE id = $1",
            endpoint_id,
        )
        .execute(&pool)
        .await
        .unwrap();
        let org_id = common::OrganizationId(common::generate_uuid_v7());
        let handler = WebhookDeliveryHandler::with_client(pool, Arc::new(cipher), Client::new());

        let outcome = handler.deliver(&due(endpoint_id, org_id)).await;

        assert!(matches!(outcome, DeliveryOutcome::Failed { .. }));
    }

    #[test]
    fn only_2xx_is_a_success() {
        assert_eq!(outcome_for(StatusCode::OK), DeliveryOutcome::Succeeded);
        assert_eq!(
            outcome_for(StatusCode::NO_CONTENT),
            DeliveryOutcome::Succeeded
        );
        assert!(matches!(
            outcome_for(StatusCode::INTERNAL_SERVER_ERROR),
            DeliveryOutcome::Failed { .. }
        ));
        assert!(matches!(
            outcome_for(StatusCode::TOO_MANY_REQUESTS),
            DeliveryOutcome::Failed { .. }
        ));
    }

    /// A redirect is a failure, not a hop. Following it would re-open the hole
    /// the guarded resolver closes.
    #[test]
    fn a_redirect_is_a_failure() {
        assert!(matches!(
            outcome_for(StatusCode::FOUND),
            DeliveryOutcome::Failed { .. }
        ));
        assert!(matches!(
            outcome_for(StatusCode::PERMANENT_REDIRECT),
            DeliveryOutcome::Failed { .. }
        ));
    }
}
