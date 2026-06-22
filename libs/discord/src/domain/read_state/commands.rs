use common::{OrganizationId, UserId};

use crate::{ChannelId, MessageId};

#[derive(Debug, Clone)]
pub struct MarkChannelReadCommand {
    pub organization_id: OrganizationId,
    pub channel_id: ChannelId,
    pub user_id: UserId,
    pub message_id: MessageId,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ChannelId, MessageId};
    use common::{OrganizationId, UserId};
    use uuid::Uuid;

    #[test]
    fn mark_channel_read_command_is_clone() {
        let cmd = MarkChannelReadCommand {
            organization_id: OrganizationId(Uuid::new_v4()),
            channel_id: ChannelId(Uuid::new_v4()),
            user_id: UserId(Uuid::new_v4()),
            message_id: MessageId(Uuid::new_v4()),
        };
        let cloned = cmd.clone();
        assert_eq!(cmd.channel_id, cloned.channel_id);
        assert_eq!(cmd.user_id, cloned.user_id);
        assert_eq!(cmd.message_id, cloned.message_id);
        assert_eq!(cmd.organization_id, cloned.organization_id);
    }
}
