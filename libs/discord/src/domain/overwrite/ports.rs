use std::future::Future;

use common::CoreError;

use crate::ChannelId;
use crate::domain::{ChannelPermissionOverwrite, OverwriteTarget};

use super::commands::UpsertChannelOverwrite;

#[cfg_attr(test, mockall::automock)]
pub trait OverwriteRepository: Send + Sync {
    fn upsert(
        &self,
        command: UpsertChannelOverwrite,
    ) -> impl Future<Output = Result<ChannelPermissionOverwrite, CoreError>> + Send;
    fn delete(
        &self,
        channel_id: ChannelId,
        target: OverwriteTarget,
    ) -> impl Future<Output = Result<(), CoreError>> + Send;
    fn list_for_channel(
        &self,
        channel_id: ChannelId,
    ) -> impl Future<Output = Result<Vec<ChannelPermissionOverwrite>, CoreError>> + Send;
}
