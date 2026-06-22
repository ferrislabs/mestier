use common::CoreError;
use discord::{
    ChannelId, CreateWebhookCommand, ExecuteWebhookCommand, Message, UpdateWebhookCommand, Webhook,
    WebhookId, WebhookRepository, WebhookService,
};
use mestier_macros::transactional;

use crate::application::MestierUseCase;

impl MestierUseCase {
    #[transactional(webhook, message)]
    pub async fn create_webhook(&self, cmd: CreateWebhookCommand) -> Result<Webhook, CoreError> {
        let org_id = cmd.organization_id;
        let mut service =
            WebhookService::new(webhook_repository, message_repository, self.events.as_ref());
        let result = service.create_webhook(cmd).await?;
        // best-effort flush at end of tx closure; events are reconciled via REST (spec §2)
        self.events.flush(org_id);
        Ok(result)
    }

    #[transactional(webhook, message)]
    pub async fn update_webhook(&self, cmd: UpdateWebhookCommand) -> Result<Webhook, CoreError> {
        let mut service =
            WebhookService::new(webhook_repository, message_repository, self.events.as_ref());
        let result = service.update_webhook(cmd).await?;
        self.events.flush(result.organization_id);
        Ok(result)
    }

    #[transactional(webhook, message)]
    pub async fn delete_webhook(&self, id: WebhookId) -> Result<(), CoreError> {
        // Load org_id before deleting so we can flush events with the correct org.
        let mut wh_repo = webhook_repository;
        let existing = wh_repo.find_by_id(id).await?.ok_or(CoreError::NotFound)?;
        let org_id = existing.organization_id;
        let mut service = WebhookService::new(wh_repo, message_repository, self.events.as_ref());
        service.delete_webhook(id).await?;
        self.events.flush(org_id);
        Ok(())
    }

    #[transactional(webhook, message)]
    pub async fn list_webhooks(&self, channel: ChannelId) -> Result<Vec<Webhook>, CoreError> {
        let mut service =
            WebhookService::new(webhook_repository, message_repository, self.events.as_ref());
        service.list_webhooks(channel).await
    }

    #[transactional(webhook)]
    pub async fn get_webhook(&self, id: WebhookId) -> Result<Webhook, CoreError> {
        let mut repo = webhook_repository;
        repo.find_by_id(id).await?.ok_or(CoreError::NotFound)
    }

    #[transactional(webhook, message)]
    pub async fn execute_webhook(&self, cmd: ExecuteWebhookCommand) -> Result<Message, CoreError> {
        let mut service =
            WebhookService::new(webhook_repository, message_repository, self.events.as_ref());
        let result = service.execute_webhook(cmd).await?;
        self.events.flush(result.organization_id);
        Ok(result)
    }
}
