use chrono::{DateTime, Utc};
use discord::{ChannelId, ChannelReadState, MessageId, OrganizationId, UserId};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct ChannelReadStateRow {
    pub user_id: Uuid,
    pub channel_id: Uuid,
    pub org_id: Uuid,
    pub last_read_message_id: Option<Uuid>,
    pub updated_at: DateTime<Utc>,
}

impl From<ChannelReadStateRow> for ChannelReadState {
    fn from(r: ChannelReadStateRow) -> Self {
        Self {
            organization_id: OrganizationId(r.org_id),
            channel_id: ChannelId(r.channel_id),
            user_id: UserId(r.user_id),
            last_read_message_id: r.last_read_message_id.map(MessageId),
            updated_at: r.updated_at,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use uuid::Uuid;

    #[test]
    fn channel_read_state_row_maps_org_id_to_organization_id() {
        let org = Uuid::from_u128(1);
        let user = Uuid::from_u128(2);
        let ch = Uuid::from_u128(3);
        let msg = Uuid::from_u128(4);
        let now = Utc::now();

        let row = ChannelReadStateRow {
            user_id: user,
            channel_id: ch,
            org_id: org,
            last_read_message_id: Some(msg),
            updated_at: now,
        };
        let state: ChannelReadState = row.into();

        assert_eq!(state.organization_id, OrganizationId(org));
        assert_eq!(state.user_id, UserId(user));
        assert_eq!(state.channel_id, ChannelId(ch));
        assert_eq!(state.last_read_message_id, Some(MessageId(msg)));
        assert_eq!(state.updated_at, now);
    }

    #[test]
    fn channel_read_state_row_maps_null_marker_to_none() {
        let now = Utc::now();
        let row = ChannelReadStateRow {
            user_id: Uuid::from_u128(1),
            channel_id: Uuid::from_u128(2),
            org_id: Uuid::from_u128(3),
            last_read_message_id: None,
            updated_at: now,
        };
        let state: ChannelReadState = row.into();
        assert!(state.last_read_message_id.is_none());
    }
}
