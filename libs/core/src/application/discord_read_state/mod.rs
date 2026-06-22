use common::CoreError;
use discord::{
    ChannelId, MarkChannelReadCommand, OrganizationId, ReadStateRepository, ReadStateService,
    UserId,
};
use mestier_macros::transactional;

use crate::application::MestierUseCase;

impl MestierUseCase {
    #[transactional(read_state, message)]
    pub async fn mark_channel_read(&self, cmd: MarkChannelReadCommand) -> Result<(), CoreError> {
        let org_id = cmd.organization_id;
        let mut service = ReadStateService::new(
            read_state_repository,
            message_repository,
            self.events.as_ref(),
        );
        service.mark_channel_read(cmd).await?;
        self.events.flush(org_id);
        Ok(())
    }

    #[transactional(read_state)]
    pub async fn list_unread_channels(
        &self,
        user_id: UserId,
        organization_id: OrganizationId,
    ) -> Result<Vec<ChannelId>, CoreError> {
        let repo = read_state_repository;
        repo.unread_channels(user_id, organization_id).await
    }
}
