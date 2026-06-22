use chrono::{DateTime, Utc};
use common::{OrganizationId, RoleId, UserId};
use serde::Serialize;

use crate::{
    components::Component,
    enums::{AuthorType, ChannelType, PresenceStatus},
    ids::{AttachmentId, CategoryId, ChannelId, MessageId, WebhookId},
};

#[derive(Debug, Clone, Serialize)]
pub struct Category {
    pub id: CategoryId,
    pub organization_id: OrganizationId,
    pub name: String,
    pub position: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Channel {
    pub id: ChannelId,
    pub organization_id: OrganizationId,
    pub channel_type: ChannelType,
    pub name: String,
    pub topic: Option<String>,
    pub position: i32,
    pub category_id: Option<CategoryId>,      // TEXT only
    pub parent_id: Option<ChannelId>,         // THREAD only
    pub origin_message_id: Option<MessageId>, // THREAD only
    pub archived: bool,                       // THREAD only (false for TEXT)
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ReactionCount {
    pub emoji: String,
    pub count: i64,
    pub user_ids: Vec<UserId>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Attachment {
    pub id: AttachmentId,
    pub storage_key: String,
    pub filename: String,
    pub mime_type: String,
    pub size_bytes: i64,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Message {
    pub id: MessageId,
    pub organization_id: OrganizationId,
    pub channel_id: ChannelId,
    pub author_type: AuthorType,
    pub author_user_id: Option<UserId>,
    pub author_webhook_id: Option<WebhookId>,
    pub content: String,
    pub components: Option<Vec<Component>>,
    pub mention_user_ids: Vec<UserId>,
    pub mention_role_ids: Vec<RoleId>,
    pub mention_channel_ids: Vec<ChannelId>,
    pub mention_everyone: bool,
    pub reactions: Vec<ReactionCount>, // aggregated when loaded
    pub attachments: Vec<Attachment>,
    pub edited_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Reaction {
    pub message_id: MessageId,
    pub emoji: String,
    pub user_id: UserId,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Webhook {
    pub id: WebhookId,
    pub organization_id: OrganizationId,
    pub channel_id: ChannelId,
    pub name: String,
    pub avatar_url: Option<String>,
    pub token: String,
    pub created_by: UserId,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Presence {
    pub organization_id: OrganizationId,
    pub user_id: UserId,
    pub status: PresenceStatus,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ChannelReadState {
    pub organization_id: OrganizationId,
    pub channel_id: ChannelId,
    pub user_id: UserId,
    pub last_read_message_id: Option<MessageId>,
    pub updated_at: DateTime<Utc>,
}

pub mod category;
pub mod channel;
pub mod message;
pub mod presence;
pub mod read_state;
pub mod webhook;
