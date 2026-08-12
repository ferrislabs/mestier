use common::CoreError;
use discord::{
    ChannelId, CreateWebhookCommand, ExecuteWebhookCommand, Message, UpdateWebhookCommand, Webhook,
    WebhookId, WebhookRepository, WebhookService,
};
use mestier_macros::transactional;

use crate::application::MestierUseCase;

impl MestierUseCase {
    #[transactional(webhook, message, events)]
    pub async fn create_webhook(&self, cmd: CreateWebhookCommand) -> Result<Webhook, CoreError> {
        let mut service = WebhookService::new(webhook_repository, message_repository, &events);
        let result = service.create_webhook(cmd).await?;
        Ok(result)
    }

    #[transactional(webhook, message, events)]
    pub async fn update_webhook(&self, cmd: UpdateWebhookCommand) -> Result<Webhook, CoreError> {
        let mut service = WebhookService::new(webhook_repository, message_repository, &events);
        let result = service.update_webhook(cmd).await?;
        Ok(result)
    }

    #[transactional(webhook, message, events)]
    pub async fn delete_webhook(&self, id: WebhookId) -> Result<(), CoreError> {
        let mut service = WebhookService::new(webhook_repository, message_repository, &events);
        service.delete_webhook(id).await?;
        Ok(())
    }

    #[transactional(webhook)]
    pub async fn list_webhooks(&self, channel: ChannelId) -> Result<Vec<Webhook>, CoreError> {
        let mut repo = webhook_repository;
        repo.list_by_channel(channel).await
    }

    #[transactional(webhook)]
    pub async fn get_webhook(&self, id: WebhookId) -> Result<Webhook, CoreError> {
        let mut repo = webhook_repository;
        repo.find_by_id(id).await?.ok_or(CoreError::NotFound)
    }

    #[transactional(webhook, message, events)]
    pub async fn execute_webhook(&self, cmd: ExecuteWebhookCommand) -> Result<Message, CoreError> {
        let mut service = WebhookService::new(webhook_repository, message_repository, &events);
        let result = service.execute_webhook(cmd).await?;
        Ok(result)
    }
}
