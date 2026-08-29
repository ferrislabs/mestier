use common::{OrganizationId, ProjectId};

use crate::{CategoryId, ChannelId, MessageId};

pub struct CreateChannelCommand {
    pub organization_id: OrganizationId,
    pub category_id: Option<CategoryId>,
    pub name: String,
    pub topic: Option<String>,
    pub position: i32,
}

/// Creates the one channel a project may have. There is no route to this
/// from the generic `POST /channels` — a project channel is always created
/// *from the project* (see `ChannelService::create_project_channel`), named
/// from it by default, which is what makes `uq_channels_project_id` and
/// `chk_channels_thread_no_project` describe a real invariant rather than an
/// accident of one call site.
pub struct CreateProjectChannelCommand {
    pub organization_id: OrganizationId,
    pub project_id: ProjectId,
    pub name: String,
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
