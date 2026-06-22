use chrono::{DateTime, Utc};
use common::CoreError;
use discord::{
    ChannelId, MessageId, Notification, NotificationId, NotificationKind, OrganizationId, UserId,
};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct NotificationRow {
    pub id: Uuid,
    pub org_id: Uuid,
    pub user_id: Uuid,
    pub channel_id: Uuid,
    pub message_id: Uuid,
    pub kind: String,
    pub read_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

impl TryFrom<NotificationRow> for Notification {
    type Error = CoreError;

    fn try_from(r: NotificationRow) -> Result<Self, CoreError> {
        let kind = r
            .kind
            .parse::<NotificationKind>()
            .map_err(|_| CoreError::Internal(format!("unknown notification kind: {}", r.kind)))?;
        Ok(Self {
            id: NotificationId(r.id),
            organization_id: OrganizationId(r.org_id),
            user_id: UserId(r.user_id),
            channel_id: ChannelId(r.channel_id),
            message_id: MessageId(r.message_id),
            kind,
            read_at: r.read_at,
            created_at: r.created_at,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use uuid::Uuid;

    fn make_row(kind: &str) -> NotificationRow {
        NotificationRow {
            id: Uuid::from_u128(1),
            org_id: Uuid::from_u128(2),
            user_id: Uuid::from_u128(3),
            channel_id: Uuid::from_u128(4),
            message_id: Uuid::from_u128(5),
            kind: kind.to_string(),
            read_at: None,
            created_at: Utc::now(),
        }
    }

    #[test]
    fn notification_row_maps_mention_kind_and_org_id() {
        let row = make_row("MENTION");
        let notif: Notification = row.try_into().unwrap();
        assert_eq!(notif.organization_id, OrganizationId(Uuid::from_u128(2)));
        assert_eq!(notif.user_id, UserId(Uuid::from_u128(3)));
        assert!(matches!(notif.kind, NotificationKind::Mention));
        assert!(notif.read_at.is_none());
    }

    #[test]
    fn notification_row_maps_reply_kind() {
        let row = make_row("REPLY");
        let notif: Notification = row.try_into().unwrap();
        assert!(matches!(notif.kind, NotificationKind::Reply));
    }

    #[test]
    fn notification_row_rejects_unknown_kind() {
        let row = make_row("UNKNOWN");
        let result: Result<Notification, _> = row.try_into();
        assert!(result.is_err());
    }
}
