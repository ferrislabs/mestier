use chrono::{DateTime, Utc};
use common::{OrganizationId, RoleId, UserId};
use serde::Serialize;

use crate::{
    components::Component,
    enums::{AuthorType, ChannelType, NotificationKind, PresenceStatus},
    ids::{AttachmentId, CategoryId, ChannelId, MessageId, NotificationId, OverwriteId, WebhookId},
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum OverwriteTarget {
    Everyone,
    Role(RoleId),
    Member(UserId),
}

#[derive(Debug, Clone, Serialize)]
pub struct ChannelPermissionOverwrite {
    pub id: OverwriteId,
    pub channel_id: ChannelId,
    pub organization_id: OrganizationId,
    pub target: OverwriteTarget,
    pub allow: i64,
    pub deny: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Notification {
    pub id: NotificationId,
    pub organization_id: OrganizationId,
    pub user_id: UserId,
    pub channel_id: ChannelId,
    pub message_id: MessageId,
    pub kind: NotificationKind,
    pub read_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[cfg(test)]
mod overwrite_tests {
    use super::*;
    use common::{RoleId, UserId};
    use uuid::Uuid;

    #[test]
    fn overwrite_target_everyone_eq() {
        assert_eq!(OverwriteTarget::Everyone, OverwriteTarget::Everyone);
    }

    #[test]
    fn overwrite_target_role_eq() {
        let role_id = RoleId(Uuid::new_v4());
        assert_eq!(
            OverwriteTarget::Role(role_id),
            OverwriteTarget::Role(role_id)
        );
    }

    #[test]
    fn overwrite_target_member_eq() {
        let user_id = UserId(Uuid::new_v4());
        assert_eq!(
            OverwriteTarget::Member(user_id),
            OverwriteTarget::Member(user_id)
        );
    }

    #[test]
    fn overwrite_target_different_variants_ne() {
        let id = Uuid::new_v4();
        assert_ne!(
            OverwriteTarget::Role(RoleId(id)),
            OverwriteTarget::Member(UserId(id)),
        );
        assert_ne!(OverwriteTarget::Everyone, OverwriteTarget::Role(RoleId(id)));
    }

    #[test]
    fn overwrite_target_is_copy() {
        let t = OverwriteTarget::Everyone;
        let _copy = t;
        let _again = t; // copy semantics — no move error
    }
}

pub mod category;
pub mod channel;
pub mod message;
pub mod notification;
pub mod overwrite;
pub mod presence;
pub mod read_state;
pub mod webhook;
