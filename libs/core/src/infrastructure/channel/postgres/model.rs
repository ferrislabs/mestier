use std::str::FromStr;

use chrono::{DateTime, Utc};
use common::CoreError;
use discord::{CategoryId, Channel, ChannelId, ChannelType, MessageId, OrganizationId};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct ChannelRow {
    pub id: Uuid,
    pub org_id: Uuid,
    pub channel_type: String,
    pub name: String,
    pub topic: Option<String>,
    pub position: i32,
    pub category_id: Option<Uuid>,
    pub parent_id: Option<Uuid>,
    pub origin_message_id: Option<Uuid>,
    pub archived: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl TryFrom<ChannelRow> for Channel {
    type Error = CoreError;

    fn try_from(r: ChannelRow) -> Result<Self, Self::Error> {
        let channel_type = ChannelType::from_str(&r.channel_type)
            .map_err(|e| CoreError::Internal(format!("invalid channel_type in db: {e}")))?;
        Ok(Self {
            id: ChannelId(r.id),
            organization_id: OrganizationId(r.org_id),
            channel_type,
            name: r.name,
            topic: r.topic,
            position: r.position,
            category_id: r.category_id.map(CategoryId),
            parent_id: r.parent_id.map(ChannelId),
            origin_message_id: r.origin_message_id.map(MessageId),
            archived: r.archived,
            created_at: r.created_at,
            updated_at: r.updated_at,
        })
    }
}
