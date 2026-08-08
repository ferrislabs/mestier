use common::CoreError;
use discord::{
    AddReactionCommand, ChannelId, CreateMessageCommand, CreateNotification, DomainEvent,
    EventPublisher, Message, MessageId, MessageRepository, MessageService, NotificationKind,
    NotificationRepository, OrganizationId, PresenceRepository, RemoveReactionCommand,
    UpdateMessageCommand, mention_notification_recipients, notification_should_deliver,
};
use mestier_macros::transactional;

use crate::application::MestierUseCase;

mod tests;

impl MestierUseCase {
    #[transactional(message, notification, presence, events)]
    pub async fn create_message(&self, cmd: CreateMessageCommand) -> Result<Message, CoreError> {
        let org_id = cmd.organization_id;
        let mut service = MessageService::new(message_repository, &events);
        let message = service.create_message(cmd).await?;

        let mut presence_repository = presence_repository;
        for recipient in mention_notification_recipients(&message) {
            let notif = notification_repository
                .create(CreateNotification {
                    organization_id: message.organization_id,
                    user_id: recipient,
                    channel_id: message.channel_id,
                    message_id: message.id,
                    kind: NotificationKind::Mention,
                })
                .await?;
            let presence = presence_repository
                .find(message.organization_id, recipient)
                .await?;
            if notification_should_deliver(presence.map(|p| p.status)) {
                events
                    .publish(DomainEvent::NotificationCreated(notif))
                    .await?;
            }
        }

        // best-effort flush at end of tx closure; events are reconciled via REST (spec §2)
        events.flush(org_id);
        Ok(message)
    }

    #[transactional(message, events)]
    pub async fn get_message(&self, id: MessageId) -> Result<Message, CoreError> {
        let mut service = MessageService::new(message_repository, &events);
        service.get_message(id).await
    }

    #[transactional(message, events)]
    pub async fn list_messages(
        &self,
        channel: ChannelId,
        before: Option<MessageId>,
        after: Option<MessageId>,
        limit: u64,
    ) -> Result<Vec<Message>, CoreError> {
        let mut service = MessageService::new(message_repository, &events);
        service.list_messages(channel, before, after, limit).await
    }

    #[transactional(message, events)]
    pub async fn update_message(&self, cmd: UpdateMessageCommand) -> Result<Message, CoreError> {
        let mut service = MessageService::new(message_repository, &events);
        let result = service.update_message(cmd).await?;
        events.flush(result.organization_id);
        Ok(result)
    }

    #[transactional(message, events)]
    pub async fn delete_message(&self, id: MessageId) -> Result<(), CoreError> {
        // Load org_id before deleting so we can flush events with the correct org.
        let mut repo = message_repository;
        let existing = repo.find_by_id(id).await?.ok_or(CoreError::NotFound)?;
        let org_id = existing.organization_id;
        let mut service = MessageService::new(repo, &events);
        service.delete_message(id).await?;
        events.flush(org_id);
        Ok(())
    }

    #[transactional(message, events)]
    pub async fn add_reaction(&self, cmd: AddReactionCommand) -> Result<(), CoreError> {
        // Load org_id from the target message to correctly scope the flush.
        let mut repo = message_repository;
        let msg = repo
            .find_by_id(cmd.message_id)
            .await?
            .ok_or(CoreError::NotFound)?;
        let org_id: OrganizationId = msg.organization_id;
        let mut service = MessageService::new(repo, &events);
        service.add_reaction(cmd).await?;
        events.flush(org_id);
        Ok(())
    }

    #[transactional(message, events)]
    pub async fn remove_reaction(&self, cmd: RemoveReactionCommand) -> Result<(), CoreError> {
        // Load org_id from the target message to correctly scope the flush.
        let mut repo = message_repository;
        let msg = repo
            .find_by_id(cmd.message_id)
            .await?
            .ok_or(CoreError::NotFound)?;
        let org_id: OrganizationId = msg.organization_id;
        let mut service = MessageService::new(repo, &events);
        service.remove_reaction(cmd).await?;
        events.flush(org_id);
        Ok(())
    }
}
