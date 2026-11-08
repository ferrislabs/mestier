use chrono::{DateTime, Utc};
use common::{CoreError, UserId};
use discord::{
    Channel, ChannelId, DirectChannel, DirectChannelRepository, DirectPair, OrganizationId,
};
use mestier_macros::repository;
use uuid::Uuid;

use crate::infrastructure::channel::postgres::model::ChannelRow;
use crate::infrastructure::postgres::{SharedTx, error::map_sqlx_error};

#[repository(domain = DirectChannel, backend = Postgres)]
pub struct PgDirectChannelRepository<'tx> {
    tx: SharedTx<'tx>,
}

impl<'tx> PgDirectChannelRepository<'tx> {
    pub fn new(tx: &SharedTx<'tx>) -> Self {
        Self { tx: tx.clone() }
    }
}

struct DirectChannelRow {
    id: Uuid,
    org_id: Uuid,
    channel_type: String,
    name: String,
    topic: Option<String>,
    position: i32,
    category_id: Option<Uuid>,
    parent_id: Option<Uuid>,
    origin_message_id: Option<Uuid>,
    archived: bool,
    project_id: Option<Uuid>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    user_low: Uuid,
    user_high: Uuid,
}

impl TryFrom<DirectChannelRow> for DirectChannel {
    type Error = CoreError;

    fn try_from(r: DirectChannelRow) -> Result<Self, Self::Error> {
        let pair = DirectPair::new(UserId(r.user_low), UserId(r.user_high))?;
        let channel = Channel::try_from(ChannelRow {
            id: r.id,
            org_id: r.org_id,
            channel_type: r.channel_type,
            name: r.name,
            topic: r.topic,
            position: r.position,
            category_id: r.category_id,
            parent_id: r.parent_id,
            origin_message_id: r.origin_message_id,
            archived: r.archived,
            project_id: r.project_id,
            created_at: r.created_at,
            updated_at: r.updated_at,
        })?;
        Ok(Self { channel, pair })
    }
}

impl<'tx> DirectChannelRepository for PgDirectChannelRepository<'tx> {
    async fn find_by_pair(
        &mut self,
        organization_id: OrganizationId,
        pair: DirectPair,
    ) -> Result<Option<DirectChannel>, CoreError> {
        let mut tx = self.tx.lock().await;
        let row = sqlx::query_as!(
            DirectChannelRow,
            r#"
            SELECT c.id, c.org_id, c.channel_type::text AS "channel_type!", c.name, c.topic,
                   c.position, c.category_id, c.parent_id, c.origin_message_id, c.archived,
                   c.project_id, c.created_at, c.updated_at, d.user_low, d.user_high
            FROM chat.direct_channels d
            JOIN chat.channels c ON c.id = d.channel_id
            WHERE d.org_id = $1 AND d.user_low = $2 AND d.user_high = $3
            "#,
            organization_id.0,
            pair.low().0,
            pair.high().0,
        )
        .fetch_optional(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;
        row.map(TryInto::try_into).transpose()
    }

    async fn find_by_channel(
        &mut self,
        channel_id: ChannelId,
    ) -> Result<Option<DirectChannel>, CoreError> {
        let mut tx = self.tx.lock().await;
        let row = sqlx::query_as!(
            DirectChannelRow,
            r#"
            SELECT c.id, c.org_id, c.channel_type::text AS "channel_type!", c.name, c.topic,
                   c.position, c.category_id, c.parent_id, c.origin_message_id, c.archived,
                   c.project_id, c.created_at, c.updated_at, d.user_low, d.user_high
            FROM chat.direct_channels d
            JOIN chat.channels c ON c.id = d.channel_id
            WHERE d.channel_id = $1
            "#,
            channel_id.0,
        )
        .fetch_optional(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;
        row.map(TryInto::try_into).transpose()
    }

    async fn insert(&mut self, channel: &Channel, pair: DirectPair) -> Result<(), CoreError> {
        let mut tx = self.tx.lock().await;
        sqlx::query!(
            r#"
            INSERT INTO chat.direct_channels (channel_id, org_id, user_low, user_high)
            VALUES ($1, $2, $3, $4)
            "#,
            channel.id.0,
            channel.organization_id.0,
            pair.low().0,
            pair.high().0,
        )
        .execute(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;
        Ok(())
    }

    async fn list_for_user(
        &mut self,
        organization_id: OrganizationId,
        user: UserId,
    ) -> Result<Vec<DirectChannel>, CoreError> {
        let mut tx = self.tx.lock().await;
        let rows = sqlx::query_as!(
            DirectChannelRow,
            r#"
            SELECT c.id, c.org_id, c.channel_type::text AS "channel_type!", c.name, c.topic,
                   c.position, c.category_id, c.parent_id, c.origin_message_id, c.archived,
                   c.project_id, c.created_at, c.updated_at, d.user_low, d.user_high
            FROM chat.direct_channels d
            JOIN chat.channels c ON c.id = d.channel_id
            WHERE d.org_id = $1 AND ($2 = d.user_low OR $2 = d.user_high)
            ORDER BY c.id DESC
            "#,
            organization_id.0,
            user.0,
        )
        .fetch_all(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;
        rows.into_iter().map(TryInto::try_into).collect()
    }
}
