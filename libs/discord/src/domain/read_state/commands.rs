use common::{OrganizationId, UserId};

use crate::{ChannelId, MessageId};

#[derive(Debug, Clone)]
pub struct MarkChannelReadCommand {
    pub organization_id: OrganizationId,
    pub channel_id: ChannelId,
    pub user_id: UserId,
    pub message_id: MessageId,
}
