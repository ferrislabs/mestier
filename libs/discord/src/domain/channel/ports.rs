use common::CoreError;

use crate::{Channel, ChannelId, OrganizationId, ProjectId};

#[cfg_attr(any(test, feature = "mock"), mockall::automock)]
pub trait ChannelRepository: Send {
    fn insert(&mut self, ch: &Channel) -> impl Future<Output = Result<Channel, CoreError>> + Send;

    fn find_by_id(
        &mut self,
        id: ChannelId,
    ) -> impl Future<Output = Result<Option<Channel>, CoreError>> + Send;

    /// The project's one channel, if it has grown one. Not filtered on
    /// `archived`: an archived project's channel must still resolve here so
    /// `restore_project` can find it again to un-archive it.
    fn find_by_project_id(
        &mut self,
        project_id: ProjectId,
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

    /// Sets `archived` in isolation, the same device `ProjectRepository::
    /// set_archived_at` uses for the project itself: a project's archive/
    /// restore cascades into its channel by calling this directly, without
    /// going through the fuller `update` (whose command never exposes
    /// `archived` for a TEXT channel — see `ChannelService::update_channel`).
    fn set_archived(
        &mut self,
        id: ChannelId,
        archived: bool,
    ) -> impl Future<Output = Result<Channel, CoreError>> + Send;

    fn delete(&mut self, id: ChannelId) -> impl Future<Output = Result<(), CoreError>> + Send;
}
