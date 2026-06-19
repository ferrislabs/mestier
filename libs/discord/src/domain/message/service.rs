use chrono::Utc;
use common::{CoreError, generate_uuid_v7};

use crate::{
    AuthorType, ChannelId, Message, MessageId, Reaction, components,
    domain::message::{
        commands::{
            AddReactionCommand, CreateMessageCommand, MessageAuthor, RemoveReactionCommand,
            UpdateMessageCommand,
        },
        ports::MessageRepository,
    },
    events::{DomainEvent, EventPublisher},
    mentions::parse_mentions,
};

/// Builds a validated, fully-populated [`Message`] from a [`CreateMessageCommand`].
///
/// Performs all invariant checks (blank content, component restrictions, component
/// shape validation) and mention parsing. Does NOT persist or publish anything.
pub(crate) fn build_new_message(cmd: CreateMessageCommand) -> Result<Message, CoreError> {
    // Invariant: components are forbidden for USER-authored messages
    let is_user = matches!(cmd.author, MessageAuthor::User(_));
    if is_user && cmd.components.is_some() {
        return Err(CoreError::Conflict(
            "components are not allowed for user-authored messages".to_owned(),
        ));
    }
    if cmd.content.trim().is_empty() {
        return Err(CoreError::Conflict(
            "message content cannot be blank".to_owned(),
        ));
    }

    if let Some(ref comps) = cmd.components {
        components::validate(comps)?;
    }

    let parsed = parse_mentions(&cmd.content);
    let (author_type, author_user_id, author_webhook_id) = match cmd.author {
        MessageAuthor::User(uid) => (AuthorType::User, Some(uid), None),
        MessageAuthor::Webhook(wid) => (AuthorType::Webhook, None, Some(wid)),
        MessageAuthor::System => (AuthorType::System, None, None),
    };

    let now = Utc::now();
    Ok(Message {
        id: MessageId(generate_uuid_v7()),
        organization_id: cmd.organization_id,
        channel_id: cmd.channel_id,
        author_type,
        author_user_id,
        author_webhook_id,
        content: cmd.content,
        components: cmd.components,
        mention_user_ids: parsed.user_ids,
        mention_role_ids: parsed.role_ids,
        mention_channel_ids: parsed.channel_ids,
        mention_everyone: parsed.everyone,
        reactions: vec![],
        edited_at: None,
        created_at: now,
    })
}

pub struct MessageService<R, E> {
    repo: R,
    events: E,
}

impl<R, E> MessageService<R, E>
where
    R: MessageRepository,
    E: EventPublisher,
{
    pub fn new(repo: R, events: E) -> Self {
        Self { repo, events }
    }

    pub async fn create_message(
        &mut self,
        cmd: CreateMessageCommand,
    ) -> Result<Message, CoreError> {
        let message = build_new_message(cmd)?;
        let saved = self.repo.insert(&message).await?;
        self.events
            .publish(DomainEvent::MessageCreated(saved.clone()))
            .await?;
        Ok(saved)
    }

    pub async fn get_message(&mut self, id: MessageId) -> Result<Message, CoreError> {
        self.repo.find_by_id(id).await?.ok_or(CoreError::NotFound)
    }

    pub async fn list_messages(
        &mut self,
        channel: ChannelId,
        before: Option<MessageId>,
        after: Option<MessageId>,
        limit: u64,
    ) -> Result<Vec<Message>, CoreError> {
        self.repo
            .list_by_channel(channel, before, after, limit)
            .await
    }

    pub async fn update_message(
        &mut self,
        cmd: UpdateMessageCommand,
    ) -> Result<Message, CoreError> {
        let existing = self.get_message(cmd.id).await?;

        // Invariant: only the original author may edit
        match existing.author_user_id {
            Some(uid) if uid == cmd.requester => {}
            _ => {
                return Err(CoreError::Conflict(
                    "only the message author may edit this message".to_owned(),
                ));
            }
        }

        if cmd.content.trim().is_empty() {
            return Err(CoreError::Conflict(
                "message content cannot be blank".to_owned(),
            ));
        }

        if let Some(ref comps) = cmd.components {
            components::validate(comps)?;
        }

        let parsed = parse_mentions(&cmd.content);
        let updated = Message {
            content: cmd.content,
            components: cmd.components,
            mention_user_ids: parsed.user_ids,
            mention_role_ids: parsed.role_ids,
            mention_channel_ids: parsed.channel_ids,
            mention_everyone: parsed.everyone,
            edited_at: Some(Utc::now()),
            ..existing
        };
        let saved = self.repo.update(&updated).await?;
        self.events
            .publish(DomainEvent::MessageUpdated(saved.clone()))
            .await?;
        Ok(saved)
    }

    pub async fn delete_message(&mut self, id: MessageId) -> Result<(), CoreError> {
        let existing = self.get_message(id).await?;
        self.repo.delete(id).await?;
        self.events
            .publish(DomainEvent::MessageDeleted {
                channel_id: existing.channel_id,
                message_id: id,
            })
            .await?;
        Ok(())
    }

    pub async fn add_reaction(&mut self, cmd: AddReactionCommand) -> Result<(), CoreError> {
        if cmd.emoji.trim().is_empty() {
            return Err(CoreError::Conflict("emoji cannot be blank".to_owned()));
        }
        let reaction = Reaction {
            message_id: cmd.message_id,
            emoji: cmd.emoji.clone(),
            user_id: cmd.user_id,
            created_at: Utc::now(),
        };
        self.repo.add_reaction(&reaction).await?;
        self.events
            .publish(DomainEvent::ReactionAdded {
                message_id: cmd.message_id,
                emoji: cmd.emoji,
                user_id: cmd.user_id,
            })
            .await?;
        Ok(())
    }

    pub async fn remove_reaction(&mut self, cmd: RemoveReactionCommand) -> Result<(), CoreError> {
        self.repo
            .remove_reaction(cmd.message_id, &cmd.emoji, cmd.user_id)
            .await?;
        self.events
            .publish(DomainEvent::ReactionRemoved {
                message_id: cmd.message_id,
                emoji: cmd.emoji,
                user_id: cmd.user_id,
            })
            .await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        AuthorType, ChannelId, MessageId, WebhookId,
        components::Component,
        domain::message::ports::MockMessageRepository,
        events::{DomainEvent, MockEventPublisher},
    };
    use common::{CoreError, OrganizationId, UserId};
    use uuid::Uuid;

    fn make_message(
        id: MessageId,
        org: OrganizationId,
        channel_id: ChannelId,
        author_user_id: Option<UserId>,
    ) -> Message {
        use chrono::Utc;
        Message {
            id,
            organization_id: org,
            channel_id,
            author_type: AuthorType::User,
            author_user_id,
            author_webhook_id: None,
            content: "Hello <@&abcdef>".to_owned(),
            components: None,
            mention_user_ids: vec![],
            mention_role_ids: vec![],
            mention_channel_ids: vec![],
            mention_everyone: false,
            reactions: vec![],
            edited_at: None,
            created_at: Utc::now(),
        }
    }

    #[tokio::test]
    async fn create_message_user_rejects_components() {
        let repo = MockMessageRepository::new();
        let events = MockEventPublisher::new();
        let mut svc = MessageService::new(repo, events);

        let result = svc
            .create_message(CreateMessageCommand {
                organization_id: OrganizationId(Uuid::new_v4()),
                channel_id: ChannelId(Uuid::new_v4()),
                author: MessageAuthor::User(UserId(Uuid::new_v4())),
                content: "hello".to_owned(),
                components: Some(vec![Component::TextDisplay {
                    content: "rich".to_owned(),
                }]),
            })
            .await;

        assert!(matches!(result, Err(CoreError::Conflict(_))));
    }

    #[tokio::test]
    async fn create_message_webhook_allows_components_and_parses_mentions() {
        let org = OrganizationId(Uuid::new_v4());
        let channel_id = ChannelId(Uuid::new_v4());
        let webhook_id = WebhookId(Uuid::new_v4());
        let user_uuid = Uuid::new_v4();

        let mut repo = MockMessageRepository::new();
        repo.expect_insert().times(1).returning(|m| {
            let m = m.clone();
            Box::pin(async move { Ok(m) })
        });

        let mut events = MockEventPublisher::new();
        events
            .expect_publish()
            .withf(|e| matches!(e, DomainEvent::MessageCreated(_)))
            .times(1)
            .returning(|_| Box::pin(async { Ok(()) }));

        let mut svc = MessageService::new(repo, events);
        let result = svc
            .create_message(CreateMessageCommand {
                organization_id: org,
                channel_id,
                author: MessageAuthor::Webhook(webhook_id),
                content: format!("hello <@{user_uuid}>"),
                components: Some(vec![Component::TextDisplay {
                    content: "embed".to_owned(),
                }]),
            })
            .await
            .unwrap();

        assert_eq!(result.author_type, AuthorType::Webhook);
        assert_eq!(result.author_webhook_id, Some(webhook_id));
        assert!(result.components.is_some());
        // mention user was parsed from content
        assert!(result.mention_user_ids.iter().any(|u| u.0 == user_uuid));
    }

    #[tokio::test]
    async fn create_message_everyone_mention_parsed() {
        let org = OrganizationId(Uuid::new_v4());
        let channel_id = ChannelId(Uuid::new_v4());

        let mut repo = MockMessageRepository::new();
        repo.expect_insert().times(1).returning(|m| {
            let m = m.clone();
            Box::pin(async move { Ok(m) })
        });

        let mut events = MockEventPublisher::new();
        events
            .expect_publish()
            .withf(|e| matches!(e, DomainEvent::MessageCreated(_)))
            .times(1)
            .returning(|_| Box::pin(async { Ok(()) }));

        let mut svc = MessageService::new(repo, events);
        let result = svc
            .create_message(CreateMessageCommand {
                organization_id: org,
                channel_id,
                author: MessageAuthor::User(UserId(Uuid::new_v4())),
                content: "hey @everyone".to_owned(),
                components: None,
            })
            .await
            .unwrap();

        assert!(result.mention_everyone);
    }

    #[tokio::test]
    async fn create_message_system_allows_components() {
        let org = OrganizationId(Uuid::new_v4());
        let channel_id = ChannelId(Uuid::new_v4());

        let mut repo = MockMessageRepository::new();
        repo.expect_insert().times(1).returning(|m| {
            let m = m.clone();
            Box::pin(async move { Ok(m) })
        });

        let mut events = MockEventPublisher::new();
        events
            .expect_publish()
            .withf(|e| matches!(e, DomainEvent::MessageCreated(_)))
            .times(1)
            .returning(|_| Box::pin(async { Ok(()) }));

        let mut svc = MessageService::new(repo, events);
        let result = svc
            .create_message(CreateMessageCommand {
                organization_id: org,
                channel_id,
                author: MessageAuthor::System,
                content: "system notice".to_owned(),
                components: Some(vec![Component::TextDisplay {
                    content: "notice body".to_owned(),
                }]),
            })
            .await
            .unwrap();

        assert_eq!(result.author_type, AuthorType::System);
        assert!(result.components.is_some());
    }

    #[tokio::test]
    async fn create_message_invalid_components_rejected() {
        let org = OrganizationId(Uuid::new_v4());
        let channel_id = ChannelId(Uuid::new_v4());

        let repo = MockMessageRepository::new();
        let events = MockEventPublisher::new();
        let mut svc = MessageService::new(repo, events);

        // Empty Container is invalid per components::validate
        let result = svc
            .create_message(CreateMessageCommand {
                organization_id: org,
                channel_id,
                author: MessageAuthor::Webhook(WebhookId(Uuid::new_v4())),
                content: "test".to_owned(),
                components: Some(vec![Component::Container {
                    accent_color: None,
                    children: vec![],
                }]),
            })
            .await;

        assert!(matches!(result, Err(CoreError::Conflict(_))));
    }

    #[tokio::test]
    async fn update_message_by_non_author_is_rejected() {
        let org = OrganizationId(Uuid::new_v4());
        let channel_id = ChannelId(Uuid::new_v4());
        let id = MessageId(Uuid::new_v4());
        let real_author = UserId(Uuid::new_v4());
        let other_user = UserId(Uuid::new_v4());

        let mut repo = MockMessageRepository::new();
        repo.expect_find_by_id()
            .with(mockall::predicate::eq(id))
            .returning(move |_| {
                Box::pin(
                    async move { Ok(Some(make_message(id, org, channel_id, Some(real_author)))) },
                )
            });

        let events = MockEventPublisher::new();
        let mut svc = MessageService::new(repo, events);

        let result = svc
            .update_message(UpdateMessageCommand {
                id,
                requester: other_user,
                content: "hacked".to_owned(),
                components: None,
            })
            .await;

        assert!(matches!(result, Err(CoreError::Conflict(_))));
    }

    #[tokio::test]
    async fn update_message_sets_edited_at_and_republishes() {
        let org = OrganizationId(Uuid::new_v4());
        let channel_id = ChannelId(Uuid::new_v4());
        let id = MessageId(Uuid::new_v4());
        let author = UserId(Uuid::new_v4());

        let mut repo = MockMessageRepository::new();
        repo.expect_find_by_id()
            .with(mockall::predicate::eq(id))
            .returning(move |_| {
                Box::pin(async move { Ok(Some(make_message(id, org, channel_id, Some(author)))) })
            });
        repo.expect_update().times(1).returning(|m| {
            let m = m.clone();
            Box::pin(async move { Ok(m) })
        });

        let mut events = MockEventPublisher::new();
        events
            .expect_publish()
            .withf(|e| matches!(e, DomainEvent::MessageUpdated(_)))
            .times(1)
            .returning(|_| Box::pin(async { Ok(()) }));

        let mut svc = MessageService::new(repo, events);
        let result = svc
            .update_message(UpdateMessageCommand {
                id,
                requester: author,
                content: "edited".to_owned(),
                components: None,
            })
            .await
            .unwrap();

        assert!(result.edited_at.is_some());
        assert_eq!(result.content, "edited");
    }

    #[tokio::test]
    async fn update_message_reparses_mentions() {
        let org = OrganizationId(Uuid::new_v4());
        let channel_id = ChannelId(Uuid::new_v4());
        let id = MessageId(Uuid::new_v4());
        let author = UserId(Uuid::new_v4());
        let mentioned = Uuid::new_v4();

        let mut repo = MockMessageRepository::new();
        repo.expect_find_by_id()
            .with(mockall::predicate::eq(id))
            .returning(move |_| {
                Box::pin(async move { Ok(Some(make_message(id, org, channel_id, Some(author)))) })
            });
        repo.expect_update().times(1).returning(|m| {
            let m = m.clone();
            Box::pin(async move { Ok(m) })
        });

        let mut events = MockEventPublisher::new();
        events
            .expect_publish()
            .withf(|e| matches!(e, DomainEvent::MessageUpdated(_)))
            .times(1)
            .returning(|_| Box::pin(async { Ok(()) }));

        let mut svc = MessageService::new(repo, events);
        let result = svc
            .update_message(UpdateMessageCommand {
                id,
                requester: author,
                content: format!("hey <@{mentioned}>"),
                components: None,
            })
            .await
            .unwrap();

        assert!(result.mention_user_ids.iter().any(|u| u.0 == mentioned));
    }

    #[tokio::test]
    async fn delete_message_publishes_message_deleted() {
        let org = OrganizationId(Uuid::new_v4());
        let channel_id = ChannelId(Uuid::new_v4());
        let id = MessageId(Uuid::new_v4());

        let mut repo = MockMessageRepository::new();
        repo.expect_find_by_id()
            .with(mockall::predicate::eq(id))
            .returning(move |_| {
                Box::pin(async move { Ok(Some(make_message(id, org, channel_id, None))) })
            });
        repo.expect_delete()
            .with(mockall::predicate::eq(id))
            .returning(|_| Box::pin(async { Ok(()) }));

        let mut events = MockEventPublisher::new();
        events
            .expect_publish()
            .withf(|e| matches!(e, DomainEvent::MessageDeleted { .. }))
            .times(1)
            .returning(|_| Box::pin(async { Ok(()) }));

        let mut svc = MessageService::new(repo, events);
        svc.delete_message(id).await.unwrap();
    }

    #[tokio::test]
    async fn delete_message_not_found_returns_error() {
        let id = MessageId(Uuid::new_v4());

        let mut repo = MockMessageRepository::new();
        repo.expect_find_by_id()
            .with(mockall::predicate::eq(id))
            .returning(|_| Box::pin(async { Ok(None) }));

        let events = MockEventPublisher::new();
        let mut svc = MessageService::new(repo, events);

        let result = svc.delete_message(id).await;
        assert!(matches!(result, Err(CoreError::NotFound)));
    }

    #[tokio::test]
    async fn add_reaction_publishes_reaction_added() {
        let message_id = MessageId(Uuid::new_v4());
        let user_id = UserId(Uuid::new_v4());
        let emoji = "👍".to_owned();

        let mut repo = MockMessageRepository::new();
        repo.expect_add_reaction()
            .times(1)
            .returning(|_| Box::pin(async { Ok(()) }));

        let mut events = MockEventPublisher::new();
        events
            .expect_publish()
            .withf(|e| matches!(e, DomainEvent::ReactionAdded { .. }))
            .times(1)
            .returning(|_| Box::pin(async { Ok(()) }));

        let mut svc = MessageService::new(repo, events);
        svc.add_reaction(AddReactionCommand {
            message_id,
            emoji: emoji.clone(),
            user_id,
        })
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn remove_reaction_publishes_reaction_removed() {
        let message_id = MessageId(Uuid::new_v4());
        let user_id = UserId(Uuid::new_v4());
        let emoji = "👍".to_owned();

        let mut repo = MockMessageRepository::new();
        repo.expect_remove_reaction()
            .times(1)
            .returning(|_, _, _| Box::pin(async { Ok(()) }));

        let mut events = MockEventPublisher::new();
        events
            .expect_publish()
            .withf(|e| matches!(e, DomainEvent::ReactionRemoved { .. }))
            .times(1)
            .returning(|_| Box::pin(async { Ok(()) }));

        let mut svc = MessageService::new(repo, events);
        svc.remove_reaction(RemoveReactionCommand {
            message_id,
            emoji: emoji.clone(),
            user_id,
        })
        .await
        .unwrap();
    }
}
