use common::CoreError;
use discord::{
    AddReactionCommand, ChannelId, ChannelRepository, ChannelType, CreateMessageCommand,
    CreateNotification, DomainEvent, EventPublisher, Message, MessageId, MessageService,
    NotificationKind, NotificationLevel, NotificationPreferenceRepository, NotificationRepository,
    PresenceRepository, RemoveReactionCommand, UpdateMessageCommand,
    mention_notification_recipients, notification_should_deliver,
};
use mestier_macros::transactional;

use crate::application::MestierUseCase;
use crate::domain::chat::events::ChatMessagePosted;

mod tests;

impl MestierUseCase {
    #[transactional(
        message,
        channel,
        notification,
        notification_preference,
        presence,
        events,
        emitter
    )]
    pub async fn create_message(&self, cmd: CreateMessageCommand) -> Result<Message, CoreError> {
        let mut service = MessageService::new(message_repository, &events);
        let message = service.create_message(cmd).await?;

        let mut presence_repository = presence_repository;
        let recipients = mention_notification_recipients(&message);
        let mut notification_preference_repository = notification_preference_repository;
        let levels = notification_preference_repository
            .list_for_channel(message.channel_id, &recipients)
            .await?;
        for recipient in recipients {
            if levels.get(&recipient) == Some(&NotificationLevel::Nothing) {
                continue;
            }
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

        let mut channel_repository = channel_repository;
        let channel = channel_repository
            .find_by_id(message.channel_id)
            .await?
            .ok_or(CoreError::NotFound)?;
        if channel.channel_type != ChannelType::Direct {
            emitter.emit(
                message.organization_id,
                &ChatMessagePosted {
                    message: message.clone(),
                },
            )?;
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
    pub async fn edit_system_message(
        &self,
        id: MessageId,
        content: String,
    ) -> Result<Message, CoreError> {
        let mut service = MessageService::new(message_repository, &events);
        service.edit_system_message(id, content).await
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
