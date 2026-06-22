use axum_extra::routing::TypedPath;
use common::OrganizationId;
use discord::{CategoryId, ChannelId, MessageId, WebhookId};
use serde::Deserialize;

// Categories
#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/chat/organizations/{organization_id}/categories")]
pub struct OrgCategoriesPath {
    pub organization_id: OrganizationId,
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/chat/categories/{category_id}")]
pub struct CategoryPath {
    pub category_id: CategoryId,
}

// Channels
#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/chat/organizations/{organization_id}/channels")]
pub struct OrgChannelsPath {
    pub organization_id: OrganizationId,
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/chat/channels/{channel_id}")]
pub struct ChannelPath {
    pub channel_id: ChannelId,
}

// Threads
#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/chat/channels/{channel_id}/threads")]
pub struct ChannelThreadsPath {
    pub channel_id: ChannelId,
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/chat/threads/{channel_id}")]
pub struct ThreadPath {
    pub channel_id: ChannelId,
}

// Messages
#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/chat/channels/{channel_id}/messages")]
pub struct ChannelMessagesPath {
    pub channel_id: ChannelId,
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/chat/messages/{message_id}")]
pub struct MessagePath {
    pub message_id: MessageId,
}

// Reactions
#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/chat/messages/{message_id}/reactions/{emoji}")]
pub struct ReactionPath {
    pub message_id: MessageId,
    pub emoji: String,
}

// Webhooks
#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/chat/channels/{channel_id}/webhooks")]
pub struct ChannelWebhooksPath {
    pub channel_id: ChannelId,
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/chat/webhooks/{webhook_id}")]
pub struct WebhookPath {
    pub webhook_id: WebhookId,
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/chat/webhooks/{webhook_id}/{token}")]
pub struct WebhookExecutePath {
    pub webhook_id: WebhookId,
    pub token: String,
}

// Presence
#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/chat/organizations/{organization_id}/presence")]
pub struct OrgPresencePath {
    pub organization_id: OrganizationId,
}

// Typing
#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/chat/channels/{channel_id}/typing")]
pub struct ChannelTypingPath {
    pub channel_id: ChannelId,
}

// Read States
#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/chat/channels/{channel_id}/read")]
pub struct ChannelReadPath {
    pub channel_id: ChannelId,
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/chat/organizations/{organization_id}/unread")]
pub struct OrgUnreadPath {
    pub organization_id: OrganizationId,
}

// Gateway (no path params)
#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/chat/gateway")]
pub struct GatewayPath;
