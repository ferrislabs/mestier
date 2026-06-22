use common::CoreError;

use crate::domain::message::ports::MessageRepository;
use crate::events::{DomainEvent, EventPublisher};

use super::commands::MarkChannelReadCommand;
use super::ports::ReadStateRepository;

pub struct ReadStateService<R, M, E> {
    read_state: R,
    messages: M,
    events: E,
}

impl<R, M, E> ReadStateService<R, M, E>
where
    R: ReadStateRepository,
    M: MessageRepository,
    E: EventPublisher,
{
    pub fn new(read_state: R, messages: M, events: E) -> Self {
        Self {
            read_state,
            messages,
            events,
        }
    }

    pub async fn mark_channel_read(
        &mut self,
        command: MarkChannelReadCommand,
    ) -> Result<(), CoreError> {
        // Step 1: validate the message exists and belongs to the command's channel
        let message = self
            .messages
            .find_by_id(command.message_id)
            .await?
            .ok_or(CoreError::NotFound)?;

        if message.channel_id != command.channel_id {
            return Err(CoreError::NotFound);
        }

        // Step 2: advance-if-greater upsert
        let moved = self.read_state.upsert(command.clone()).await?;

        // Step 3: publish only when the marker actually moved
        if let Some(state) = moved {
            self.events
                .publish(DomainEvent::ChannelRead {
                    organization_id: state.organization_id,
                    channel_id: state.channel_id,
                    user_id: state.user_id,
                    last_read_message_id: state.last_read_message_id,
                })
                .await?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::ChannelReadState;
    use crate::domain::message::ports::MockMessageRepository;
    use crate::domain::read_state::commands::MarkChannelReadCommand;
    use crate::domain::read_state::ports::MockReadStateRepository;
    use crate::events::MockEventPublisher;
    use crate::{ChannelId, MessageId};
    use chrono::Utc;
    use common::{CoreError, OrganizationId, UserId};
    use uuid::Uuid;

    fn make_message(
        message_id: crate::MessageId,
        channel_id: ChannelId,
        org_id: OrganizationId,
    ) -> crate::domain::Message {
        use crate::domain::Message;
        use crate::enums::AuthorType;
        Message {
            id: message_id,
            organization_id: org_id,
            channel_id,
            author_type: AuthorType::User,
            author_user_id: Some(UserId(Uuid::new_v4())),
            author_webhook_id: None,
            content: "hello".to_owned(),
            components: None,
            mention_user_ids: vec![],
            mention_role_ids: vec![],
            mention_channel_ids: vec![],
            mention_everyone: false,
            reactions: vec![],
            attachments: vec![],
            edited_at: None,
            created_at: Utc::now(),
        }
    }

    /// Advancing the marker: message exists in the right channel, upsert returns Some →
    /// DomainEvent::ChannelRead must be published exactly once.
    #[tokio::test]
    async fn mark_channel_read_advance_stores_and_publishes() {
        let org_id = OrganizationId(Uuid::new_v4());
        let channel_id = ChannelId(Uuid::new_v4());
        let user_id = UserId(Uuid::new_v4());
        let message_id = MessageId(Uuid::new_v4());

        let msg = make_message(message_id, channel_id, org_id);

        let mut messages = MockMessageRepository::new();
        messages
            .expect_find_by_id()
            .withf(move |id| *id == message_id)
            .times(1)
            .returning(move |_| {
                let m = msg.clone();
                Box::pin(async move { Ok(Some(m)) })
            });

        let state = ChannelReadState {
            organization_id: org_id,
            channel_id,
            user_id,
            last_read_message_id: Some(message_id),
            updated_at: Utc::now(),
        };

        let mut read_state = MockReadStateRepository::new();
        read_state.expect_upsert().times(1).returning(move |_| {
            let s = state.clone();
            Box::pin(async move { Ok(Some(s)) })
        });

        let mut events = MockEventPublisher::new();
        events
            .expect_publish()
            .withf(move |e| match e {
                DomainEvent::ChannelRead {
                    organization_id: oid,
                    channel_id: cid,
                    user_id: uid,
                    ..
                } => *oid == org_id && *cid == channel_id && *uid == user_id,
                _ => false,
            })
            .times(1)
            .returning(|_| Box::pin(async { Ok(()) }));

        let mut svc = ReadStateService::new(read_state, messages, events);
        let result = svc
            .mark_channel_read(MarkChannelReadCommand {
                organization_id: org_id,
                channel_id,
                user_id,
                message_id,
            })
            .await;

        assert!(result.is_ok());
    }

    /// Re-acking an older or equal message id: upsert returns None → no event published.
    #[tokio::test]
    async fn mark_channel_read_older_ack_is_noop_no_publish() {
        let org_id = OrganizationId(Uuid::new_v4());
        let channel_id = ChannelId(Uuid::new_v4());
        let user_id = UserId(Uuid::new_v4());
        let message_id = MessageId(Uuid::new_v4());

        let msg = make_message(message_id, channel_id, org_id);

        let mut messages = MockMessageRepository::new();
        messages.expect_find_by_id().times(1).returning(move |_| {
            let m = msg.clone();
            Box::pin(async move { Ok(Some(m)) })
        });

        let mut read_state = MockReadStateRepository::new();
        read_state
            .expect_upsert()
            .times(1)
            .returning(|_| Box::pin(async { Ok(None) }));

        let mut events = MockEventPublisher::new();
        events.expect_publish().times(0);

        let mut svc = ReadStateService::new(read_state, messages, events);
        let result = svc
            .mark_channel_read(MarkChannelReadCommand {
                organization_id: org_id,
                channel_id,
                user_id,
                message_id,
            })
            .await;

        assert!(result.is_ok());
    }

    /// Message belongs to a different channel → CoreError::NotFound, upsert NOT called.
    #[tokio::test]
    async fn mark_channel_read_message_from_other_channel_returns_not_found() {
        let org_id = OrganizationId(Uuid::new_v4());
        let command_channel = ChannelId(Uuid::new_v4());
        let message_channel = ChannelId(Uuid::new_v4()); // different
        let user_id = UserId(Uuid::new_v4());
        let message_id = MessageId(Uuid::new_v4());

        // Message's channel_id does NOT match command.channel_id
        let msg = make_message(message_id, message_channel, org_id);

        let mut messages = MockMessageRepository::new();
        messages.expect_find_by_id().times(1).returning(move |_| {
            let m = msg.clone();
            Box::pin(async move { Ok(Some(m)) })
        });

        // upsert must never be called
        let read_state = MockReadStateRepository::new();
        let events = MockEventPublisher::new();

        let mut svc = ReadStateService::new(read_state, messages, events);
        let result = svc
            .mark_channel_read(MarkChannelReadCommand {
                organization_id: org_id,
                channel_id: command_channel,
                user_id,
                message_id,
            })
            .await;

        assert!(matches!(result, Err(CoreError::NotFound)));
    }

    /// Message not found (find_by_id returns Ok(None)) → CoreError::NotFound,
    /// upsert and publish must NOT be called.
    #[tokio::test]
    async fn mark_channel_read_message_not_found_returns_not_found() {
        let org_id = OrganizationId(Uuid::new_v4());
        let channel_id = ChannelId(Uuid::new_v4());
        let user_id = UserId(Uuid::new_v4());
        let message_id = MessageId(Uuid::new_v4());

        let mut messages = MockMessageRepository::new();
        messages
            .expect_find_by_id()
            .times(1)
            .returning(|_| Box::pin(async { Ok(None) }));

        let mut read_state = MockReadStateRepository::new();
        read_state.expect_upsert().times(0);

        let mut events = MockEventPublisher::new();
        events.expect_publish().times(0);

        let mut svc = ReadStateService::new(read_state, messages, events);
        let result = svc
            .mark_channel_read(MarkChannelReadCommand {
                organization_id: org_id,
                channel_id,
                user_id,
                message_id,
            })
            .await;

        assert!(matches!(result, Err(CoreError::NotFound)));
    }
}
