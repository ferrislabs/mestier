use common::CoreError;
use discord::{
    Channel, ChannelId, ChannelRepository, ChannelService, CreateChannelCommand,
    CreateThreadCommand, OrganizationId, UpdateChannelCommand, UpdateThreadCommand,
};
use mestier_macros::transactional;

use crate::application::MestierUseCase;

impl MestierUseCase {
    #[transactional(channel, events)]
    pub async fn create_channel(&self, cmd: CreateChannelCommand) -> Result<Channel, CoreError> {
        let mut service = ChannelService::new(channel_repository, &events);
        let result = service.create_channel(cmd).await?;
        Ok(result)
    }

    #[transactional(channel, events)]
    pub async fn update_channel(&self, cmd: UpdateChannelCommand) -> Result<Channel, CoreError> {
        let mut service = ChannelService::new(channel_repository, &events);
        let result = service.update_channel(cmd).await?;
        Ok(result)
    }

    #[transactional(channel, events)]
    pub async fn delete_channel(&self, id: ChannelId) -> Result<(), CoreError> {
        let mut service = ChannelService::new(channel_repository, &events);
        service.delete_channel(id).await?;
        Ok(())
    }

    #[transactional(channel, events)]
    pub async fn list_channels(&self, org: OrganizationId) -> Result<Vec<Channel>, CoreError> {
        let mut service = ChannelService::new(channel_repository, &events);
        service.list_channels(org).await
    }

    #[transactional(channel)]
    pub async fn get_channel(&self, id: ChannelId) -> Result<Channel, CoreError> {
        let mut repo = channel_repository;
        repo.find_by_id(id).await?.ok_or(CoreError::NotFound)
    }

    #[transactional(channel, events)]
    pub async fn create_thread(&self, cmd: CreateThreadCommand) -> Result<Channel, CoreError> {
        let mut service = ChannelService::new(channel_repository, &events);
        let result = service.create_thread(cmd).await?;
        Ok(result)
    }

    #[transactional(channel, events)]
    pub async fn update_thread(&self, cmd: UpdateThreadCommand) -> Result<Channel, CoreError> {
        let mut service = ChannelService::new(channel_repository, &events);
        let result = service.update_thread(cmd).await?;
        Ok(result)
    }

    #[transactional(channel, events)]
    pub async fn delete_thread(&self, id: ChannelId) -> Result<(), CoreError> {
        let mut service = ChannelService::new(channel_repository, &events);
        service.delete_thread(id).await?;
        Ok(())
    }

    #[transactional(channel, events)]
    pub async fn list_threads(&self, parent: ChannelId) -> Result<Vec<Channel>, CoreError> {
        let mut service = ChannelService::new(channel_repository, &events);
        service.list_threads(parent).await
    }
}
