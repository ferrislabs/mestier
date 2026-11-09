use std::collections::HashSet;

use common::CoreError;
use discord::{
    CategoryId, CategoryRepository, Channel, ChannelId, ChannelPlacement, ChannelRepository,
    ChannelService, CreateChannelCommand, CreateThreadCommand, OrganizationId,
    UpdateChannelCommand, UpdateThreadCommand,
};
use mestier_macros::transactional;

use crate::application::MestierUseCase;
use crate::domain::chat::events::ChatChannelCreated;

#[cfg(test)]
mod tests;

impl MestierUseCase {
    #[transactional(channel, events, emitter)]
    pub async fn create_channel(&self, cmd: CreateChannelCommand) -> Result<Channel, CoreError> {
        let mut service = ChannelService::new(channel_repository, &events);
        let result = service.create_channel(cmd).await?;
        emitter.emit(
            result.organization_id,
            &ChatChannelCreated {
                channel: result.clone(),
            },
        )?;
        Ok(result)
    }

    /// Moves channels between categories and ranks them, in one transaction.
    #[transactional(channel, category, events)]
    pub async fn reorder_channels(
        &self,
        organization_id: OrganizationId,
        placements: Vec<ChannelPlacement>,
    ) -> Result<Vec<Channel>, CoreError> {
        let mut category_repository = category_repository;
        let categories: HashSet<CategoryId> = category_repository
            .list_by_organization(organization_id)
            .await?
            .into_iter()
            .map(|category| category.id)
            .collect();
        let mut service = ChannelService::new(channel_repository, &events);
        service
            .reorder_channels(organization_id, &placements, &categories)
            .await
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
