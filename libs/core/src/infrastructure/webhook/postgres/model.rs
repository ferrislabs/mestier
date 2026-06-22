use chrono::{DateTime, Utc};
use discord::{ChannelId, OrganizationId, UserId, Webhook, WebhookId};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct WebhookRow {
    pub id: Uuid,
    pub org_id: Uuid,
    pub channel_id: Uuid,
    pub name: String,
    pub avatar_url: Option<String>,
    pub token: String,
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<WebhookRow> for Webhook {
    fn from(r: WebhookRow) -> Self {
        Self {
            id: WebhookId(r.id),
            organization_id: OrganizationId(r.org_id),
            channel_id: ChannelId(r.channel_id),
            name: r.name,
            avatar_url: r.avatar_url,
            token: r.token,
            created_by: UserId(r.created_by),
            created_at: r.created_at,
            updated_at: r.updated_at,
        }
    }
}
