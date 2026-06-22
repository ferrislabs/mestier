use common::{CoreError, OrganizationId, UserId};

use crate::{Category, CategoryId, Channel, ChannelId, Message, MessageId, PresenceStatus};

#[derive(Debug, Clone)]
pub enum DomainEvent {
    MessageCreated(Message),
    MessageUpdated(Message),
    MessageDeleted {
        channel_id: ChannelId,
        message_id: MessageId,
    },
    ReactionAdded {
        message_id: MessageId,
        emoji: String,
        user_id: UserId,
    },
    ReactionRemoved {
        message_id: MessageId,
        emoji: String,
        user_id: UserId,
    },
    CategoryCreated(Category),
    CategoryUpdated(Category),
    CategoryDeleted {
        organization_id: OrganizationId,
        category_id: CategoryId,
    },
    ChannelCreated(Channel),
    ChannelUpdated(Channel),
    ChannelDeleted {
        organization_id: OrganizationId,
        channel_id: ChannelId,
    },
    ThreadCreated(Channel),
    ThreadUpdated(Channel),
    ThreadDeleted {
        organization_id: OrganizationId,
        channel_id: ChannelId,
    },
    PresenceUpdated {
        organization_id: OrganizationId,
        user_id: UserId,
        status: PresenceStatus,
    },
    TypingStarted {
        organization_id: OrganizationId,
        channel_id: ChannelId,
        user_id: UserId,
    },
    ChannelRead {
        organization_id: OrganizationId,
        channel_id: ChannelId,
        user_id: UserId,
        last_read_message_id: Option<MessageId>,
    },
}

#[cfg_attr(test, mockall::automock)]
pub trait EventPublisher: Send + Sync {
    fn publish(&self, event: DomainEvent) -> impl Future<Output = Result<(), CoreError>> + Send;
}

#[cfg(test)]
mod tests {
    use super::*;
    use common::{OrganizationId, UserId};
    use uuid::Uuid;

    #[tokio::test]
    async fn mock_event_publisher_accepts_typing_started() {
        let mut publisher = MockEventPublisher::new();
        publisher
            .expect_publish()
            .withf(|e| matches!(e, DomainEvent::TypingStarted { .. }))
            .times(1)
            .returning(|_| Box::pin(async { Ok(()) }));

        publisher
            .publish(DomainEvent::TypingStarted {
                organization_id: OrganizationId(Uuid::new_v4()),
                channel_id: crate::ChannelId(Uuid::new_v4()),
                user_id: UserId(Uuid::new_v4()),
            })
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn mock_event_publisher_accepts_channel_read() {
        let mut publisher = MockEventPublisher::new();
        publisher
            .expect_publish()
            .withf(|e| matches!(e, DomainEvent::ChannelRead { .. }))
            .times(1)
            .returning(|_| Box::pin(async { Ok(()) }));

        publisher
            .publish(DomainEvent::ChannelRead {
                organization_id: OrganizationId(Uuid::new_v4()),
                channel_id: crate::ChannelId(Uuid::new_v4()),
                user_id: UserId(Uuid::new_v4()),
                last_read_message_id: None,
            })
            .await
            .unwrap();
    }
}
