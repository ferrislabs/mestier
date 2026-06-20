use common::CoreError;

use crate::{Channel, ChannelId, OrganizationId};

#[cfg_attr(test, mockall::automock)]
pub trait ChannelRepository: Send {
    fn insert(&mut self, ch: &Channel) -> impl Future<Output = Result<Channel, CoreError>> + Send;

    fn find_by_id(
        &mut self,
        id: ChannelId,
    ) -> impl Future<Output = Result<Option<Channel>, CoreError>> + Send;

    fn list_by_organization(
        &mut self,
        org: OrganizationId,
    ) -> impl Future<Output = Result<Vec<Channel>, CoreError>> + Send;

    fn list_threads(
        &mut self,
        parent: ChannelId,
    ) -> impl Future<Output = Result<Vec<Channel>, CoreError>> + Send;

    fn update(&mut self, ch: &Channel) -> impl Future<Output = Result<Channel, CoreError>> + Send;

    fn delete(&mut self, id: ChannelId) -> impl Future<Output = Result<(), CoreError>> + Send;
}
