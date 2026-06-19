use chrono::Utc;
use common::{CoreError, generate_uuid_v7};
use uuid::Uuid;

use crate::{
    ChannelId, Webhook, WebhookId,
    domain::{
        message::{
            commands::{CreateMessageCommand, MessageAuthor},
            ports::MessageRepository,
            service::build_new_message,
        },
        webhook::{
            commands::{CreateWebhookCommand, ExecuteWebhookCommand, UpdateWebhookCommand},
            ports::WebhookRepository,
        },
    },
    events::{DomainEvent, EventPublisher},
};

pub struct WebhookService<R, MR, E> {
    repo: R,
    message_repo: MR,
    events: E,
}

impl<R, MR, E> WebhookService<R, MR, E>
where
    R: WebhookRepository,
    MR: MessageRepository,
    E: EventPublisher,
{
    pub fn new(repo: R, message_repo: MR, events: E) -> Self {
        Self {
            repo,
            message_repo,
            events,
        }
    }

    pub async fn create_webhook(
        &mut self,
        cmd: CreateWebhookCommand,
    ) -> Result<Webhook, CoreError> {
        if cmd.name.trim().is_empty() {
            return Err(CoreError::Conflict(
                "webhook name cannot be blank".to_owned(),
            ));
        }
        let now = Utc::now();
        // Generate a random opaque token (32 hex chars, UUID v4 simple form)
        let token = Uuid::new_v4().simple().to_string();
        let wh = Webhook {
            id: WebhookId(generate_uuid_v7()),
            organization_id: cmd.organization_id,
            channel_id: cmd.channel_id,
            name: cmd.name,
            avatar_url: cmd.avatar_url,
            token,
            created_by: cmd.created_by,
            created_at: now,
            updated_at: now,
        };
        self.repo.insert(&wh).await
    }

    pub async fn update_webhook(
        &mut self,
        cmd: UpdateWebhookCommand,
    ) -> Result<Webhook, CoreError> {
        if cmd.name.trim().is_empty() {
            return Err(CoreError::Conflict(
                "webhook name cannot be blank".to_owned(),
            ));
        }
        let existing = self
            .repo
            .find_by_id(cmd.id)
            .await?
            .ok_or(CoreError::NotFound)?;
        let updated = Webhook {
            name: cmd.name,
            avatar_url: cmd.avatar_url,
            updated_at: Utc::now(),
            ..existing
        };
        let saved = self.repo.update(&updated).await?;
        Ok(saved)
    }

    pub async fn delete_webhook(&mut self, id: WebhookId) -> Result<(), CoreError> {
        let _existing = self.repo.find_by_id(id).await?.ok_or(CoreError::NotFound)?;
        self.repo.delete(id).await?;
        Ok(())
    }

    pub async fn list_webhooks(&mut self, channel: ChannelId) -> Result<Vec<Webhook>, CoreError> {
        self.repo.list_by_channel(channel).await
    }

    pub async fn execute_webhook(
        &mut self,
        cmd: ExecuteWebhookCommand,
    ) -> Result<crate::Message, CoreError> {
        let webhook = self
            .repo
            .find_by_id(cmd.webhook_id)
            .await?
            .ok_or(CoreError::NotFound)?;

        // Invariant: the caller must supply the correct token
        if webhook.token != cmd.token {
            return Err(CoreError::Forbidden { reason: None });
        }

        let message = build_new_message(CreateMessageCommand {
            organization_id: webhook.organization_id,
            channel_id: webhook.channel_id,
            author: MessageAuthor::Webhook(cmd.webhook_id),
            content: cmd.content,
            components: cmd.components,
        })?;
        let saved = self.message_repo.insert(&message).await?;
        self.events
            .publish(DomainEvent::MessageCreated(saved.clone()))
            .await?;
        Ok(saved)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        AuthorType, ChannelId, WebhookId,
        components::Component,
        domain::{message::ports::MockMessageRepository, webhook::ports::MockWebhookRepository},
        events::{DomainEvent, MockEventPublisher},
    };
    use common::{CoreError, OrganizationId, UserId};
    use uuid::Uuid;

    fn make_webhook(id: WebhookId, org: OrganizationId, channel_id: ChannelId) -> Webhook {
        use chrono::Utc;
        Webhook {
            id,
            organization_id: org,
            channel_id,
            name: "My Bot".to_owned(),
            avatar_url: None,
            token: "secret-token".to_owned(),
            created_by: UserId(Uuid::new_v4()),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    #[tokio::test]
    async fn execute_webhook_bad_token_is_rejected() {
        let org = OrganizationId(Uuid::new_v4());
        let channel_id = ChannelId(Uuid::new_v4());
        let wh_id = WebhookId(Uuid::new_v4());

        let mut wh_repo = MockWebhookRepository::new();
        wh_repo
            .expect_find_by_id()
            .with(mockall::predicate::eq(wh_id))
            .returning(move |_| {
                Box::pin(async move { Ok(Some(make_webhook(wh_id, org, channel_id))) })
            });

        let mut msg_repo = MockMessageRepository::new();
        msg_repo.expect_insert().times(0);

        let mut events = MockEventPublisher::new();
        events.expect_publish().times(0);

        let mut svc = WebhookService::new(wh_repo, msg_repo, events);

        let result = svc
            .execute_webhook(ExecuteWebhookCommand {
                webhook_id: wh_id,
                token: "wrong-token".to_owned(),
                content: "hello".to_owned(),
                components: None,
            })
            .await;

        assert!(matches!(result, Err(CoreError::Forbidden { .. })));
    }

    #[tokio::test]
    async fn execute_webhook_rejects_blank_content() {
        let org = OrganizationId(Uuid::new_v4());
        let channel_id = ChannelId(Uuid::new_v4());
        let wh_id = WebhookId(Uuid::new_v4());

        let mut wh_repo = MockWebhookRepository::new();
        wh_repo
            .expect_find_by_id()
            .with(mockall::predicate::eq(wh_id))
            .returning(move |_| {
                Box::pin(async move { Ok(Some(make_webhook(wh_id, org, channel_id))) })
            });

        let mut msg_repo = MockMessageRepository::new();
        msg_repo.expect_insert().times(0);

        let mut events = MockEventPublisher::new();
        events.expect_publish().times(0);

        let mut svc = WebhookService::new(wh_repo, msg_repo, events);

        let result = svc
            .execute_webhook(ExecuteWebhookCommand {
                webhook_id: wh_id,
                token: "secret-token".to_owned(),
                content: "   ".to_owned(),
                components: None,
            })
            .await;

        assert!(matches!(result, Err(CoreError::Conflict(_))));
    }

    #[tokio::test]
    async fn execute_webhook_correct_token_creates_webhook_authored_message() {
        let org = OrganizationId(Uuid::new_v4());
        let channel_id = ChannelId(Uuid::new_v4());
        let wh_id = WebhookId(Uuid::new_v4());

        let mut wh_repo = MockWebhookRepository::new();
        wh_repo
            .expect_find_by_id()
            .with(mockall::predicate::eq(wh_id))
            .returning(move |_| {
                Box::pin(async move { Ok(Some(make_webhook(wh_id, org, channel_id))) })
            });

        let mut msg_repo = MockMessageRepository::new();
        msg_repo.expect_insert().times(1).returning(|m| {
            let m = m.clone();
            Box::pin(async move { Ok(m) })
        });

        let mut events = MockEventPublisher::new();
        events
            .expect_publish()
            .withf(|e| matches!(e, DomainEvent::MessageCreated(_)))
            .times(1)
            .returning(|_| Box::pin(async { Ok(()) }));

        let mut svc = WebhookService::new(wh_repo, msg_repo, events);
        let msg = svc
            .execute_webhook(ExecuteWebhookCommand {
                webhook_id: wh_id,
                token: "secret-token".to_owned(),
                content: "hello from webhook".to_owned(),
                components: Some(vec![Component::TextDisplay {
                    content: "embed".to_owned(),
                }]),
            })
            .await
            .unwrap();

        assert_eq!(msg.author_type, AuthorType::Webhook);
        assert_eq!(msg.author_webhook_id, Some(wh_id));
        assert!(msg.components.is_some());
    }

    #[tokio::test]
    async fn execute_webhook_not_found_returns_error() {
        let wh_id = WebhookId(Uuid::new_v4());

        let mut wh_repo = MockWebhookRepository::new();
        wh_repo
            .expect_find_by_id()
            .with(mockall::predicate::eq(wh_id))
            .returning(|_| Box::pin(async { Ok(None) }));

        let msg_repo = MockMessageRepository::new();
        let events = MockEventPublisher::new();
        let mut svc = WebhookService::new(wh_repo, msg_repo, events);

        let result = svc
            .execute_webhook(ExecuteWebhookCommand {
                webhook_id: wh_id,
                token: "any-token".to_owned(),
                content: "hello".to_owned(),
                components: None,
            })
            .await;

        assert!(matches!(result, Err(CoreError::NotFound)));
    }

    #[tokio::test]
    async fn create_webhook_persists_and_generates_token() {
        let org = OrganizationId(Uuid::new_v4());
        let channel_id = ChannelId(Uuid::new_v4());
        let created_by = UserId(Uuid::new_v4());

        let mut wh_repo = MockWebhookRepository::new();
        wh_repo.expect_insert().times(1).returning(|w| {
            let w = w.clone();
            Box::pin(async move { Ok(w) })
        });

        let msg_repo = MockMessageRepository::new();
        let mut events = MockEventPublisher::new();
        events.expect_publish().times(0);

        let mut svc = WebhookService::new(wh_repo, msg_repo, events);
        let result = svc
            .create_webhook(CreateWebhookCommand {
                organization_id: org,
                channel_id,
                name: "My Bot".to_owned(),
                avatar_url: None,
                created_by,
            })
            .await
            .unwrap();

        assert_eq!(result.organization_id, org);
        assert!(!result.token.is_empty(), "token must be generated");
    }

    #[tokio::test]
    async fn create_webhook_blank_name_is_rejected() {
        let org = OrganizationId(Uuid::new_v4());
        let channel_id = ChannelId(Uuid::new_v4());
        let created_by = UserId(Uuid::new_v4());

        let wh_repo = MockWebhookRepository::new();
        let msg_repo = MockMessageRepository::new();
        let events = MockEventPublisher::new();
        let mut svc = WebhookService::new(wh_repo, msg_repo, events);

        let result = svc
            .create_webhook(CreateWebhookCommand {
                organization_id: org,
                channel_id,
                name: "   ".to_owned(),
                avatar_url: None,
                created_by,
            })
            .await;

        assert!(matches!(result, Err(CoreError::Conflict(_))));
    }

    #[tokio::test]
    async fn update_webhook_not_found_returns_error() {
        let wh_id = WebhookId(Uuid::new_v4());

        let mut wh_repo = MockWebhookRepository::new();
        wh_repo
            .expect_find_by_id()
            .with(mockall::predicate::eq(wh_id))
            .returning(|_| Box::pin(async { Ok(None) }));

        let msg_repo = MockMessageRepository::new();
        let events = MockEventPublisher::new();
        let mut svc = WebhookService::new(wh_repo, msg_repo, events);

        let result = svc
            .update_webhook(UpdateWebhookCommand {
                id: wh_id,
                name: "New Name".to_owned(),
                avatar_url: None,
            })
            .await;

        assert!(matches!(result, Err(CoreError::NotFound)));
    }

    #[tokio::test]
    async fn update_webhook_returns_updated_webhook_without_event() {
        let org = OrganizationId(Uuid::new_v4());
        let channel_id = ChannelId(Uuid::new_v4());
        let wh_id = WebhookId(Uuid::new_v4());

        let mut wh_repo = MockWebhookRepository::new();
        wh_repo
            .expect_find_by_id()
            .with(mockall::predicate::eq(wh_id))
            .times(1)
            .returning(move |_| {
                Box::pin(async move { Ok(Some(make_webhook(wh_id, org, channel_id))) })
            });
        wh_repo.expect_update().times(1).returning(|w| {
            let w = w.clone();
            Box::pin(async move { Ok(w) })
        });

        let msg_repo = MockMessageRepository::new();
        let mut events = MockEventPublisher::new();
        events.expect_publish().times(0);

        let mut svc = WebhookService::new(wh_repo, msg_repo, events);
        let result = svc
            .update_webhook(UpdateWebhookCommand {
                id: wh_id,
                name: "Renamed Bot".to_owned(),
                avatar_url: Some("https://example.com/avatar.png".to_owned()),
            })
            .await
            .unwrap();

        assert_eq!(result.name, "Renamed Bot");
    }

    #[tokio::test]
    async fn delete_webhook_not_found_returns_error() {
        let wh_id = WebhookId(Uuid::new_v4());

        let mut wh_repo = MockWebhookRepository::new();
        wh_repo
            .expect_find_by_id()
            .with(mockall::predicate::eq(wh_id))
            .returning(|_| Box::pin(async { Ok(None) }));

        let msg_repo = MockMessageRepository::new();
        let events = MockEventPublisher::new();
        let mut svc = WebhookService::new(wh_repo, msg_repo, events);

        let result = svc.delete_webhook(wh_id).await;
        assert!(matches!(result, Err(CoreError::NotFound)));
    }

    #[tokio::test]
    async fn delete_webhook_deletes_without_publishing_event() {
        let org = OrganizationId(Uuid::new_v4());
        let channel_id = ChannelId(Uuid::new_v4());
        let wh_id = WebhookId(Uuid::new_v4());

        let mut wh_repo = MockWebhookRepository::new();
        wh_repo
            .expect_find_by_id()
            .with(mockall::predicate::eq(wh_id))
            .times(1)
            .returning(move |_| {
                Box::pin(async move { Ok(Some(make_webhook(wh_id, org, channel_id))) })
            });
        wh_repo
            .expect_delete()
            .with(mockall::predicate::eq(wh_id))
            .times(1)
            .returning(|_| Box::pin(async { Ok(()) }));

        let msg_repo = MockMessageRepository::new();
        let mut events = MockEventPublisher::new();
        events.expect_publish().times(0);

        let mut svc = WebhookService::new(wh_repo, msg_repo, events);
        svc.delete_webhook(wh_id).await.unwrap();
    }
}
