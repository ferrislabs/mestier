use common::{OrganizationId, UserId};

use crate::{ChannelId, MessageId, NotificationKind};

#[derive(Debug, Clone)]
pub struct CreateNotification {
    pub organization_id: OrganizationId,
    pub user_id: UserId,
    pub channel_id: ChannelId,
    pub message_id: MessageId,
    pub kind: NotificationKind,
}

#[derive(Debug, Clone)]
pub struct MarkNotificationRead {
    pub notification_id: crate::NotificationId,
    pub user_id: UserId,
}
