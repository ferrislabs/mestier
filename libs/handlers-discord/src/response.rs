use chrono::{DateTime, Utc};
use common::{OrganizationId, RoleId, UserId};
use discord::components::Component;
use discord::{
    Attachment, AuthorType, Category, CategoryId, Channel, ChannelId, ChannelType, Message,
    MessageId, Presence, PresenceStatus, ReactionCount, Webhook, WebhookId,
};
use serde::Serialize;
use utoipa::ToSchema;

// ── Category ──────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Serialize, ToSchema)]
pub struct CategoryResponse {
    pub id: CategoryId,
    pub organization_id: OrganizationId,
    pub name: String,
    pub position: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<Category> for CategoryResponse {
    fn from(c: Category) -> Self {
        Self {
            id: c.id,
            organization_id: c.organization_id,
            name: c.name,
            position: c.position,
            created_at: c.created_at,
            updated_at: c.updated_at,
        }
    }
}

// ── Channel ───────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Serialize, ToSchema)]
pub struct ChannelResponse {
    pub id: ChannelId,
    pub organization_id: OrganizationId,
    pub channel_type: ChannelType,
    pub name: String,
    pub topic: Option<String>,
    pub position: i32,
    pub category_id: Option<CategoryId>,
    /// Thread only: parent text channel.
    pub parent_id: Option<ChannelId>,
    /// Thread only: message that spawned this thread.
    pub origin_message_id: Option<MessageId>,
    /// Thread only: whether the thread is archived.
    pub archived: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<Channel> for ChannelResponse {
    fn from(c: Channel) -> Self {
        Self {
            id: c.id,
            organization_id: c.organization_id,
            channel_type: c.channel_type,
            name: c.name,
            topic: c.topic,
            position: c.position,
            category_id: c.category_id,
            parent_id: c.parent_id,
            origin_message_id: c.origin_message_id,
            archived: c.archived,
            created_at: c.created_at,
            updated_at: c.updated_at,
        }
    }
}

// ── Reaction count ────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Serialize, ToSchema)]
pub struct ReactionCountResponse {
    pub emoji: String,
    pub count: i64,
    pub user_ids: Vec<UserId>,
}

impl From<ReactionCount> for ReactionCountResponse {
    fn from(r: ReactionCount) -> Self {
        Self {
            emoji: r.emoji,
            count: r.count,
            user_ids: r.user_ids,
        }
    }
}

// ── Attachment ────────────────────────────────────────────────────────────────

/// Per-message attachment. The internal `AttachmentId` is deliberately omitted —
/// clients reference files by `storage_key` via the download endpoint.
#[derive(Debug, Clone, PartialEq, Serialize, ToSchema)]
pub struct AttachmentResponse {
    pub storage_key: String,
    pub filename: String,
    pub mime_type: String,
    pub size_bytes: i64,
}

impl From<Attachment> for AttachmentResponse {
    fn from(a: Attachment) -> Self {
        Self {
            storage_key: a.storage_key,
            filename: a.filename,
            mime_type: a.mime_type,
            size_bytes: a.size_bytes,
        }
    }
}

// ── Message ───────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Serialize, ToSchema)]
pub struct MessageResponse {
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
    pub reactions: Vec<ReactionCountResponse>,
    pub attachments: Vec<AttachmentResponse>,
    pub edited_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

impl From<Message> for MessageResponse {
    fn from(m: Message) -> Self {
        Self {
            id: m.id,
            organization_id: m.organization_id,
            channel_id: m.channel_id,
            author_type: m.author_type,
            author_user_id: m.author_user_id,
            author_webhook_id: m.author_webhook_id,
            content: m.content,
            components: m.components,
            mention_user_ids: m.mention_user_ids,
            mention_role_ids: m.mention_role_ids,
            mention_channel_ids: m.mention_channel_ids,
            mention_everyone: m.mention_everyone,
            reactions: m
                .reactions
                .into_iter()
                .map(ReactionCountResponse::from)
                .collect(),
            attachments: m
                .attachments
                .into_iter()
                .map(AttachmentResponse::from)
                .collect(),
            edited_at: m.edited_at,
            created_at: m.created_at,
        }
    }
}

// ── Webhook ───────────────────────────────────────────────────────────────────

/// Public webhook view returned by list/get endpoints.
///
/// The `token` field is intentionally absent — it must never be returned
/// outside the single create-webhook response (use [`webhook_response_with_token`]).
#[derive(Debug, Clone, PartialEq, Serialize, ToSchema)]
pub struct WebhookResponse {
    pub id: WebhookId,
    pub organization_id: OrganizationId,
    pub channel_id: ChannelId,
    pub name: String,
    pub avatar_url: Option<String>,
    pub created_by: UserId,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<Webhook> for WebhookResponse {
    fn from(w: Webhook) -> Self {
        // NOTE: `w.token` is deliberately dropped here — it must not leak.
        Self {
            id: w.id,
            organization_id: w.organization_id,
            channel_id: w.channel_id,
            name: w.name,
            avatar_url: w.avatar_url,
            created_by: w.created_by,
            created_at: w.created_at,
            updated_at: w.updated_at,
        }
    }
}

/// Webhook creation response — the ONLY response that includes the token.
///
/// Returned once on `POST /webhooks`; token is never surfaced again.
#[derive(Debug, Clone, PartialEq, Serialize, ToSchema)]
pub struct WebhookCreatedResponse {
    pub id: WebhookId,
    pub organization_id: OrganizationId,
    pub channel_id: ChannelId,
    pub name: String,
    pub avatar_url: Option<String>,
    /// Signing token — returned once at creation time only.
    pub token: String,
    pub created_by: UserId,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Converts a freshly-created [`Webhook`] into the one-time creation response
/// that includes the token.  Do NOT use this for get/list endpoints.
pub fn webhook_created_response(w: Webhook) -> WebhookCreatedResponse {
    WebhookCreatedResponse {
        id: w.id,
        organization_id: w.organization_id,
        channel_id: w.channel_id,
        name: w.name,
        avatar_url: w.avatar_url,
        token: w.token,
        created_by: w.created_by,
        created_at: w.created_at,
        updated_at: w.updated_at,
    }
}

// ── Presence ──────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Serialize, ToSchema)]
pub struct PresenceResponse {
    pub organization_id: OrganizationId,
    pub user_id: UserId,
    pub status: PresenceStatus,
    pub updated_at: DateTime<Utc>,
}

impl From<Presence> for PresenceResponse {
    fn from(p: Presence) -> Self {
        Self {
            organization_id: p.organization_id,
            user_id: p.user_id,
            status: p.status,
            updated_at: p.updated_at,
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::*;
    use chrono::Utc;
    use common::OrganizationId;
    use discord::{Category, CategoryId};

    // Static UUIDs — avoids pulling in the `uuid` crate directly.
    const UUID_A: &str = "550e8400-e29b-41d4-a716-446655440000";
    const UUID_B: &str = "6ba7b810-9dad-11d1-80b4-00c04fd430c8";
    const UUID_C: &str = "6ba7b811-9dad-11d1-80b4-00c04fd430c8";
    const UUID_D: &str = "6ba7b812-9dad-11d1-80b4-00c04fd430c8";

    #[test]
    fn category_response_from_domain() {
        let id = CategoryId::from_str(UUID_A).unwrap();
        let org = OrganizationId::from_str(UUID_B).unwrap();
        let now = Utc::now();
        let cat = Category {
            id,
            organization_id: org,
            name: "General".into(),
            position: 0,
            created_at: now,
            updated_at: now,
        };
        let resp = CategoryResponse::from(cat);
        assert_eq!(resp.name, "General");
        assert_eq!(resp.position, 0);
    }

    #[test]
    fn webhook_response_does_not_serialize_token() {
        let webhook = Webhook {
            id: WebhookId::from_str(UUID_A).unwrap(),
            organization_id: OrganizationId::from_str(UUID_B).unwrap(),
            channel_id: ChannelId::from_str(UUID_C).unwrap(),
            name: "ci-bot".into(),
            avatar_url: None,
            token: "super-secret-token".into(),
            created_by: UserId::from_str(UUID_D).unwrap(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        let resp = WebhookResponse::from(webhook);
        let json = serde_json::to_string(&resp).expect("serialize");
        assert!(
            !json.contains("token"),
            "WebhookResponse must not expose the token field; got: {json}"
        );
    }

    #[test]
    fn attachment_response_does_not_expose_id() {
        use discord::{Attachment, AttachmentId};

        let attachment = Attachment {
            id: AttachmentId::from_str(UUID_A).unwrap(),
            storage_key: "prefix/attachments/018f1234".to_owned(),
            filename: "invoice.pdf".to_owned(),
            mime_type: "application/pdf".to_owned(),
            size_bytes: 204_800,
            created_at: Utc::now(),
        };
        let resp = AttachmentResponse::from(attachment);
        let json = serde_json::to_string(&resp).expect("serialize");
        assert!(
            !json.contains("id"),
            "AttachmentResponse must not expose id; got: {json}"
        );
        assert!(
            json.contains("invoice.pdf"),
            "filename must be present; got: {json}"
        );
        assert!(
            json.contains("204800"),
            "size_bytes must be present; got: {json}"
        );
    }

    #[test]
    fn message_response_carries_attachments() {
        use discord::{Attachment, AttachmentId, AuthorType, ChannelId, Message, MessageId};

        let msg_id = MessageId::from_str(UUID_A).unwrap();
        let org_id = OrganizationId::from_str(UUID_B).unwrap();
        let ch_id = ChannelId::from_str(UUID_C).unwrap();
        let att_id = AttachmentId::from_str(UUID_D).unwrap();
        let now = Utc::now();

        let message = Message {
            id: msg_id,
            organization_id: org_id,
            channel_id: ch_id,
            author_type: AuthorType::User,
            author_user_id: None,
            author_webhook_id: None,
            content: "see attached".to_owned(),
            components: None,
            mention_user_ids: vec![],
            mention_role_ids: vec![],
            mention_channel_ids: vec![],
            mention_everyone: false,
            reactions: vec![],
            attachments: vec![Attachment {
                id: att_id,
                storage_key: "prefix/attachments/abc123".to_owned(),
                filename: "photo.png".to_owned(),
                mime_type: "image/png".to_owned(),
                size_bytes: 51_200,
                created_at: now,
            }],
            edited_at: None,
            created_at: now,
        };

        let resp = MessageResponse::from(message);
        assert_eq!(resp.attachments.len(), 1);
        assert_eq!(resp.attachments[0].filename, "photo.png");
        assert_eq!(resp.attachments[0].storage_key, "prefix/attachments/abc123");
        assert_eq!(resp.attachments[0].size_bytes, 51_200);
    }
}
