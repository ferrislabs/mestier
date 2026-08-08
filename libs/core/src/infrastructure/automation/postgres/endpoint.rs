use common::{CoreError, OrganizationId};
use mestier_macros::repository;
use uuid::Uuid;

use crate::{
    domain::automation::{
        endpoint::{SealedSecret, WebhookEndpoint},
        ports::WebhookEndpointRepository,
    },
    infrastructure::postgres::{SharedTx, error::map_sqlx_error},
};

#[repository(domain = WebhookEndpoint, backend = Postgres)]
pub struct PgWebhookEndpointRepository<'tx> {
    tx: SharedTx<'tx>,
}

impl<'tx> PgWebhookEndpointRepository<'tx> {
    pub fn new(tx: &SharedTx<'tx>) -> Self {
        Self { tx: tx.clone() }
    }
}

macro_rules! endpoint_from {
    ($row:expr) => {
        WebhookEndpoint {
            id: $row.id,
            org_id: OrganizationId($row.org_id),
            url: $row.url,
            description: $row.description,
            enabled: $row.enabled,
            created_at: $row.created_at,
            updated_at: $row.updated_at,
            disabled_at: $row.disabled_at,
        }
    };
}

impl<'tx> WebhookEndpointRepository for PgWebhookEndpointRepository<'tx> {
    async fn insert(
        &mut self,
        endpoint: &WebhookEndpoint,
        secret: &SealedSecret,
    ) -> Result<WebhookEndpoint, CoreError> {
        let mut tx = self.tx.lock().await;
        let row = sqlx::query!(
            r#"INSERT INTO automation.webhook_endpoint
                   (id, org_id, url, secret_nonce, secret_ciphertext, description, enabled)
               VALUES ($1, $2, $3, $4, $5, $6, true)
               RETURNING id, org_id, url, description, enabled, created_at, updated_at, disabled_at"#,
            endpoint.id,
            endpoint.org_id.0,
            endpoint.url,
            secret.nonce,
            secret.ciphertext,
            endpoint.description,
        )
        .fetch_one(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        Ok(endpoint_from!(row))
    }

    async fn list_by_organization(
        &mut self,
        org_id: OrganizationId,
    ) -> Result<Vec<WebhookEndpoint>, CoreError> {
        let mut tx = self.tx.lock().await;
        let rows = sqlx::query!(
            r#"SELECT id, org_id, url, description, enabled, created_at, updated_at, disabled_at
               FROM automation.webhook_endpoint
               WHERE org_id = $1
               ORDER BY created_at DESC"#,
            org_id.0,
        )
        .fetch_all(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        Ok(rows.into_iter().map(|row| endpoint_from!(row)).collect())
    }

    async fn find_by_id(&mut self, id: Uuid) -> Result<Option<WebhookEndpoint>, CoreError> {
        let mut tx = self.tx.lock().await;
        let row = sqlx::query!(
            r#"SELECT id, org_id, url, description, enabled, created_at, updated_at, disabled_at
               FROM automation.webhook_endpoint WHERE id = $1"#,
            id,
        )
        .fetch_optional(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        Ok(row.map(|row| endpoint_from!(row)))
    }

    async fn update(&mut self, endpoint: &WebhookEndpoint) -> Result<WebhookEndpoint, CoreError> {
        let mut tx = self.tx.lock().await;
        let row = sqlx::query!(
            r#"UPDATE automation.webhook_endpoint
               SET url = $2,
                   description = $3,
                   enabled = $4,
                   disabled_at = CASE WHEN $4 THEN NULL ELSE COALESCE(disabled_at, now()) END,
                   updated_at = now()
               WHERE id = $1
               RETURNING id, org_id, url, description, enabled, created_at, updated_at, disabled_at"#,
            endpoint.id,
            endpoint.url,
            endpoint.description,
            endpoint.enabled,
        )
        .fetch_one(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        Ok(endpoint_from!(row))
    }

    async fn reseal(&mut self, id: Uuid, secret: &SealedSecret) -> Result<(), CoreError> {
        let mut tx = self.tx.lock().await;
        sqlx::query!(
            r#"UPDATE automation.webhook_endpoint
               SET secret_nonce = $2, secret_ciphertext = $3, updated_at = now()
               WHERE id = $1"#,
            id,
            secret.nonce,
            secret.ciphertext,
        )
        .execute(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        Ok(())
    }

    async fn delete(&mut self, id: Uuid) -> Result<(), CoreError> {
        let mut tx = self.tx.lock().await;
        sqlx::query!("DELETE FROM automation.webhook_endpoint WHERE id = $1", id)
            .execute(&mut ***tx)
            .await
            .map_err(map_sqlx_error)?;

        Ok(())
    }
}
