use common::CoreError;
use discord::{
    AddReactionCommand, ChannelId, CreateMessageCommand, CreateNotification, DomainEvent,
    EventPublisher, Message, MessageId, MessageService, NotificationKind, NotificationRepository,
    PresenceRepository, RemoveReactionCommand, UpdateMessageCommand,
    mention_notification_recipients, notification_should_deliver,
};
use mestier_macros::transactional;

use crate::application::MestierUseCase;

mod tests;

impl MestierUseCase {
    #[transactional(message, notification, presence, events)]
    pub async fn create_message(&self, cmd: CreateMessageCommand) -> Result<Message, CoreError> {
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
        Ok(result)
    }

    #[transactional(message, events)]
    pub async fn delete_message(&self, id: MessageId) -> Result<(), CoreError> {
        let mut service = MessageService::new(message_repository, &events);
        service.delete_message(id).await?;
        Ok(())
    }

    #[transactional(message, events)]
    pub async fn add_reaction(&self, cmd: AddReactionCommand) -> Result<(), CoreError> {
        let mut service = MessageService::new(message_repository, &events);
        service.add_reaction(cmd).await?;
        Ok(())
    }

    #[transactional(message, events)]
    pub async fn remove_reaction(&self, cmd: RemoveReactionCommand) -> Result<(), CoreError> {
        let mut service = MessageService::new(message_repository, &events);
        service.remove_reaction(cmd).await?;
        Ok(())
    }
}
