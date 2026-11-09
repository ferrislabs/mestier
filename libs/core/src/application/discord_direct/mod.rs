use common::CoreError;
use discord::{DirectChannel, DirectChannelRepository, DirectChannelService};
use mestier_macros::transactional;

use crate::{
    UserId,
    application::MestierUseCase,
    domain::{member::ports::MemberRepository, organization::OrganizationId},
};

#[cfg(test)]
mod tests;

impl MestierUseCase {
    /// The conversation between `caller` and `peer`, created on first use.
    /// `peer` must hold a seat in the organization: a direct message is
    /// between two members, never to an outsider.
    #[transactional(channel, direct_channel, member)]
    pub async fn open_direct_channel(
        &self,
        organization_id: OrganizationId,
        caller: UserId,
        peer: UserId,
    ) -> Result<DirectChannel, CoreError> {
        let mut member_repository = member_repository;
        member_repository
            .find_by_org_and_user(organization_id, peer)
            .await?
            .ok_or(CoreError::NotFound)?;
        let mut service = DirectChannelService::new(channel_repository, direct_channel_repository);
        service.open(organization_id, caller, peer).await
    }

    #[transactional(direct_channel)]
    pub async fn list_direct_channels(
        &self,
        organization_id: OrganizationId,
        user: UserId,
    ) -> Result<Vec<DirectChannel>, CoreError> {
        let mut repo = direct_channel_repository;
        repo.list_for_user(organization_id, user).await
    }
}
