use common::{OrganizationId, UserId};

use crate::{ChannelId, WebhookId, components::Component};

pub struct CreateWebhookCommand {
    pub organization_id: OrganizationId,
    pub channel_id: ChannelId,
    pub name: String,
    pub avatar_url: Option<String>,
    pub created_by: UserId,
}

pub struct UpdateWebhookCommand {
    pub id: WebhookId,
    pub name: String,
    pub avatar_url: Option<String>,
}

pub struct ExecuteWebhookCommand {
    pub webhook_id: WebhookId,
    /// The secret token the caller must provide (checked against `Webhook::token`).
    pub token: String,
    pub content: String,
    pub components: Option<Vec<Component>>,
}
