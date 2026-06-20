use common::OrganizationId;

use crate::{CategoryId, ChannelId, MessageId};

pub struct CreateChannelCommand {
    pub organization_id: OrganizationId,
    pub category_id: Option<CategoryId>,
    pub name: String,
    pub topic: Option<String>,
    pub position: i32,
}

pub struct UpdateChannelCommand {
    pub id: ChannelId,
    pub category_id: Option<CategoryId>,
    pub name: String,
    pub topic: Option<String>,
    pub position: i32,
}

pub struct CreateThreadCommand {
    pub organization_id: OrganizationId,
    pub parent_id: ChannelId,
    pub origin_message_id: Option<MessageId>,
    pub name: String,
}

pub struct UpdateThreadCommand {
    pub id: ChannelId,
    pub name: String,
    pub archived: bool,
}
