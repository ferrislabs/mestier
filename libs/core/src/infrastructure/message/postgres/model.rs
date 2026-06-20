use std::str::FromStr;

use chrono::{DateTime, Utc};
use common::CoreError;
use discord::{
    AuthorType, ChannelId, Message, MessageId, OrganizationId, Reaction, ReactionCount, RoleId,
    UserId, WebhookId, components::Component,
};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct MessageRow {
    pub id: Uuid,
    pub org_id: Uuid,
    pub channel_id: Uuid,
    pub author_type: String,
    pub author_user_id: Option<Uuid>,
    pub author_webhook_id: Option<Uuid>,
    pub content: String,
    pub components: Option<serde_json::Value>,
    pub mention_user_ids: Vec<Uuid>,
    pub mention_role_ids: Vec<Uuid>,
    pub mention_channel_ids: Vec<Uuid>,
    pub mention_everyone: bool,
    pub edited_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

impl MessageRow {
    pub fn into_message(self, reactions: Vec<ReactionCount>) -> Result<Message, CoreError> {
        let author_type = AuthorType::from_str(&self.author_type)
            .map_err(|e| CoreError::Internal(format!("invalid author_type in db: {e}")))?;
        let components: Option<Vec<Component>> = match self.components {
            Some(v) => Some(
                serde_json::from_value(v)
                    .map_err(|e| CoreError::Internal(format!("invalid components json: {e}")))?,
            ),
            None => None,
        };
        Ok(Message {
            id: MessageId(self.id),
            organization_id: OrganizationId(self.org_id),
            channel_id: ChannelId(self.channel_id),
            author_type,
            author_user_id: self.author_user_id.map(UserId),
            author_webhook_id: self.author_webhook_id.map(WebhookId),
            content: self.content,
            components,
            mention_user_ids: self.mention_user_ids.into_iter().map(UserId).collect(),
            mention_role_ids: self.mention_role_ids.into_iter().map(RoleId).collect(),
            mention_channel_ids: self
                .mention_channel_ids
                .into_iter()
                .map(ChannelId)
                .collect(),
            mention_everyone: self.mention_everyone,
            reactions,
            edited_at: self.edited_at,
            created_at: self.created_at,
        })
    }
}

#[derive(Debug, Clone)]
pub struct ReactionRow {
    pub message_id: Uuid,
    pub emoji: String,
    pub user_id: Uuid,
    pub created_at: DateTime<Utc>,
}

impl From<ReactionRow> for Reaction {
    fn from(r: ReactionRow) -> Self {
        Self {
            message_id: MessageId(r.message_id),
            emoji: r.emoji,
            user_id: UserId(r.user_id),
            created_at: r.created_at,
        }
    }
}

// Aggregated row from a GROUP BY query
#[derive(Debug, Clone)]
pub struct ReactionAggRow {
    pub emoji: String,
    pub count: i64,
    pub user_ids: Vec<Uuid>,
}

impl From<ReactionAggRow> for ReactionCount {
    fn from(r: ReactionAggRow) -> Self {
        Self {
            emoji: r.emoji,
            count: r.count,
            user_ids: r.user_ids.into_iter().map(UserId).collect(),
        }
    }
}
