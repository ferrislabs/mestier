use common::{OrganizationId, UserId};

use crate::{ChannelId, MessageId, WebhookId, components::Component};

/// Identifies the author for `CreateMessageCommand`.
pub enum MessageAuthor {
    User(UserId),
    Webhook(WebhookId),
    System,
}

pub struct CreateMessageCommand {
    pub organization_id: OrganizationId,
    pub channel_id: ChannelId,
    pub author: MessageAuthor,
    pub content: String,
    pub components: Option<Vec<Component>>,
}

pub struct UpdateMessageCommand {
    pub id: MessageId,
    /// The user requesting the edit — must match `author_user_id`.
    pub requester: UserId,
    pub content: String,
    pub components: Option<Vec<Component>>,
}

pub struct AddReactionCommand {
    pub message_id: MessageId,
    pub emoji: String,
    pub user_id: UserId,
}

pub struct RemoveReactionCommand {
    pub message_id: MessageId,
    pub emoji: String,
    pub user_id: UserId,
}
