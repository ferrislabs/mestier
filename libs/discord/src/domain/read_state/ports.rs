use std::future::Future;

use common::{CoreError, OrganizationId, UserId};

use crate::{ChannelId, domain::ChannelReadState};

use super::commands::MarkChannelReadCommand;

#[cfg_attr(test, mockall::automock)]
pub trait ReadStateRepository: Send + Sync {
    fn upsert(
        &self,
        command: MarkChannelReadCommand,
    ) -> impl Future<Output = Result<Option<ChannelReadState>, CoreError>> + Send;
    fn get(
        &self,
        user_id: UserId,
        channel_id: ChannelId,
    ) -> impl Future<Output = Result<Option<ChannelReadState>, CoreError>> + Send;
    fn unread_channels(
        &self,
        user_id: UserId,
        organization_id: OrganizationId,
    ) -> impl Future<Output = Result<Vec<ChannelId>, CoreError>> + Send;
}
