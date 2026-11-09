use std::collections::HashMap;

use common::CoreError;
use discord::{ChannelId, NotificationLevel, OrganizationId, UserId};
use mestier_macros::repository;

use crate::infrastructure::postgres::{SharedTx, error::map_sqlx_error};

#[repository(domain = NotificationPreference, backend = Postgres)]
pub struct PgNotificationPreferenceRepository<'tx> {
    tx: SharedTx<'tx>,
}

impl<'tx> PgNotificationPreferenceRepository<'tx> {
    pub fn new(tx: &SharedTx<'tx>) -> Self {
        Self { tx: tx.clone() }
    }
}

impl<'tx> discord::NotificationPreferenceRepository for PgNotificationPreferenceRepository<'tx> {
    async fn upsert(
        &mut self,
        organization_id: OrganizationId,
        user_id: UserId,
        channel_id: ChannelId,
        level: NotificationLevel,
    ) -> Result<(), CoreError> {
        let mut tx = self.tx.lock().await;
        sqlx::query!(
            r#"
            INSERT INTO chat.notification_preference (user_id, channel_id, org_id, level, updated_at)
            VALUES ($1, $2, $3, $4, now())
            ON CONFLICT (user_id, channel_id)
            DO UPDATE SET level = EXCLUDED.level, updated_at = now()
            "#,
            user_id.0,
            channel_id.0,
            organization_id.0,
            level.as_str(),
        )
        .execute(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;
        Ok(())
    }

    async fn list_for_user(
        &mut self,
        organization_id: OrganizationId,
        user_id: UserId,
    ) -> Result<HashMap<ChannelId, NotificationLevel>, CoreError> {
        let mut tx = self.tx.lock().await;
        let rows = sqlx::query!(
            r#"
            SELECT channel_id, level
            FROM chat.notification_preference
            WHERE org_id = $1 AND user_id = $2
            "#,
            organization_id.0,
            user_id.0,
        )
        .fetch_all(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        rows.into_iter()
            .map(|row| {
                let level = row
                    .level
                    .parse::<NotificationLevel>()
                    .map_err(CoreError::Internal)?;
                Ok((ChannelId(row.channel_id), level))
            })
            .collect()
    }

    async fn list_for_channel(
        &mut self,
        channel_id: ChannelId,
        user_ids: &[UserId],
    ) -> Result<HashMap<UserId, NotificationLevel>, CoreError> {
        let ids: Vec<uuid::Uuid> = user_ids.iter().map(|user| user.0).collect();
        let mut tx = self.tx.lock().await;
        let rows = sqlx::query!(
            r#"
            SELECT user_id, level
            FROM chat.notification_preference
            WHERE channel_id = $1 AND user_id = ANY($2)
            "#,
            channel_id.0,
            &ids,
        )
        .fetch_all(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        rows.into_iter()
            .map(|row| {
                let level = row
                    .level
                    .parse::<NotificationLevel>()
                    .map_err(CoreError::Internal)?;
                Ok((UserId(row.user_id), level))
            })
            .collect()
    }
}
