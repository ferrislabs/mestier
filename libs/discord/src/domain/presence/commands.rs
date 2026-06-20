use common::{OrganizationId, UserId};

use crate::{ChannelId, PresenceStatus};

pub struct SetPresenceCommand {
    pub organization_id: OrganizationId,
    pub user_id: UserId,
    pub status: PresenceStatus,
}

pub struct StartTypingCommand {
    pub organization_id: OrganizationId,
    pub channel_id: ChannelId,
    pub user_id: UserId,
}
