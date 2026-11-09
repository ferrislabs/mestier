use common::{CoreError, OrganizationId, UserId};

use crate::{Channel, ChannelId};

use super::{DirectChannel, DirectPair};

#[cfg_attr(any(test, feature = "mock"), mockall::automock)]
pub trait DirectChannelRepository: Send {
    fn find_by_pair(
        &mut self,
        organization_id: OrganizationId,
        pair: DirectPair,
    ) -> impl Future<Output = Result<Option<DirectChannel>, CoreError>> + Send;

    fn find_by_channel(
        &mut self,
        channel_id: ChannelId,
    ) -> impl Future<Output = Result<Option<DirectChannel>, CoreError>> + Send;

    /// Records who a freshly inserted `DIRECT` channel belongs to.
    fn insert(
        &mut self,
        channel: &Channel,
        pair: DirectPair,
    ) -> impl Future<Output = Result<(), CoreError>> + Send;

    /// Every direct channel `user` is part of, most recently opened first.
    fn list_for_user(
        &mut self,
        organization_id: OrganizationId,
        user: UserId,
    ) -> impl Future<Output = Result<Vec<DirectChannel>, CoreError>> + Send;
}
