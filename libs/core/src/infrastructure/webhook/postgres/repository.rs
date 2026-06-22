use common::CoreError;
use discord::domain::webhook::ports::WebhookRepository;
use discord::{ChannelId, Webhook, WebhookId};
use mestier_macros::repository;

use super::model::WebhookRow;
use crate::infrastructure::postgres::{SharedTx, error::map_sqlx_error};

#[repository(domain = Webhook, backend = Postgres)]
pub struct PgWebhookRepository<'tx> {
    tx: SharedTx<'tx>,
}

impl<'tx> PgWebhookRepository<'tx> {
    pub fn new(tx: &SharedTx<'tx>) -> Self {
        Self { tx: tx.clone() }
    }
}

impl<'tx> WebhookRepository for PgWebhookRepository<'tx> {
    async fn insert(&mut self, w: &Webhook) -> Result<Webhook, CoreError> {
        let mut tx = self.tx.lock().await;
        let row = sqlx::query_as!(
			WebhookRow,
			r#"
			INSERT INTO chat.webhooks (id, org_id, channel_id, name, avatar_url, token, created_by, created_at, updated_at)
			VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
			RETURNING id, org_id, channel_id, name, avatar_url, token, created_by, created_at, updated_at
			"#,
			w.id.0,
			w.organization_id.0,
			w.channel_id.0,
			w.name,
			w.avatar_url,
			w.token,
			w.created_by.0,
			w.created_at,
			w.updated_at,
		)
		.fetch_one(&mut ***tx)
		.await
		.map_err(map_sqlx_error)?;
        Ok(row.into())
    }

    async fn find_by_id(&mut self, id: WebhookId) -> Result<Option<Webhook>, CoreError> {
        let mut tx = self.tx.lock().await;
        let row = sqlx::query_as!(
            WebhookRow,
            r#"
			SELECT id, org_id, channel_id, name, avatar_url, token, created_by, created_at, updated_at
			FROM chat.webhooks WHERE id = $1
			"#,
            id.0,
        )
        .fetch_optional(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;
        Ok(row.map(Into::into))
    }

    async fn list_by_channel(&mut self, channel: ChannelId) -> Result<Vec<Webhook>, CoreError> {
        let mut tx = self.tx.lock().await;
        let rows = sqlx::query_as!(
            WebhookRow,
            r#"
			SELECT id, org_id, channel_id, name, avatar_url, token, created_by, created_at, updated_at
			FROM chat.webhooks WHERE channel_id = $1 ORDER BY created_at ASC
			"#,
            channel.0,
        )
        .fetch_all(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;
        Ok(rows.into_iter().map(Into::into).collect())
    }

    async fn update(&mut self, w: &Webhook) -> Result<Webhook, CoreError> {
        let mut tx = self.tx.lock().await;
        let row = sqlx::query_as!(
            WebhookRow,
            r#"
			UPDATE chat.webhooks SET name = $2, avatar_url = $3, updated_at = $4 WHERE id = $1
			RETURNING id, org_id, channel_id, name, avatar_url, token, created_by, created_at, updated_at
			"#,
            w.id.0,
            w.name,
            w.avatar_url,
            w.updated_at,
        )
        .fetch_optional(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;
        row.map(Into::into).ok_or(CoreError::NotFound)
    }

    async fn delete(&mut self, id: WebhookId) -> Result<(), CoreError> {
        let mut tx = self.tx.lock().await;
        let result = sqlx::query!("DELETE FROM chat.webhooks WHERE id = $1", id.0)
            .execute(&mut ***tx)
            .await
            .map_err(map_sqlx_error)?;
        if result.rows_affected() == 0 {
            return Err(CoreError::NotFound);
        }
        Ok(())
    }
}
