use chrono::Utc;
use common::{CoreError, generate_uuid_v7};

use crate::{
    Channel, ChannelId, ChannelType, OrganizationId,
    domain::channel::{
        commands::{
            CreateChannelCommand, CreateThreadCommand, UpdateChannelCommand, UpdateThreadCommand,
        },
        ports::ChannelRepository,
    },
    events::{DomainEvent, EventPublisher},
};

pub struct ChannelService<R, E> {
    repo: R,
    events: E,
}

impl<R, E> ChannelService<R, E>
where
    R: ChannelRepository,
    E: EventPublisher,
{
    pub fn new(repo: R, events: E) -> Self {
        Self { repo, events }
    }

    pub async fn create_channel(
        &mut self,
        cmd: CreateChannelCommand,
    ) -> Result<Channel, CoreError> {
        if cmd.name.trim().is_empty() {
            return Err(CoreError::Conflict(
                "channel name cannot be blank".to_owned(),
            ));
        }
        let now = Utc::now();
        let ch = Channel {
            id: ChannelId(generate_uuid_v7()),
            organization_id: cmd.organization_id,
            channel_type: ChannelType::Text,
            name: cmd.name,
            topic: cmd.topic,
            position: cmd.position,
            category_id: cmd.category_id,
            parent_id: None,
            origin_message_id: None,
            archived: false,
            created_at: now,
            updated_at: now,
        };
        let saved = self.repo.insert(&ch).await?;
        self.events
            .publish(DomainEvent::ChannelCreated(saved.clone()))
            .await?;
        Ok(saved)
    }

    pub async fn update_channel(
        &mut self,
        cmd: UpdateChannelCommand,
    ) -> Result<Channel, CoreError> {
        if cmd.name.trim().is_empty() {
            return Err(CoreError::Conflict(
                "channel name cannot be blank".to_owned(),
            ));
        }
        let existing = self
            .repo
            .find_by_id(cmd.id)
            .await?
            .ok_or(CoreError::NotFound)?;
        let updated = Channel {
            id: existing.id,
            organization_id: existing.organization_id,
            channel_type: existing.channel_type,
            name: cmd.name,
            topic: cmd.topic,
            position: cmd.position,
            category_id: cmd.category_id,
            parent_id: existing.parent_id,
            origin_message_id: existing.origin_message_id,
            archived: existing.archived,
            created_at: existing.created_at,
            updated_at: Utc::now(),
        };
        let saved = self.repo.update(&updated).await?;
        self.events
            .publish(DomainEvent::ChannelUpdated(saved.clone()))
            .await?;
        Ok(saved)
    }

    pub async fn delete_channel(&mut self, id: ChannelId) -> Result<(), CoreError> {
        let existing = self.repo.find_by_id(id).await?.ok_or(CoreError::NotFound)?;
        self.repo.delete(id).await?;
        self.events
            .publish(DomainEvent::ChannelDeleted {
                organization_id: existing.organization_id,
                channel_id: id,
            })
            .await?;
        Ok(())
    }

    pub async fn list_channels(&mut self, org: OrganizationId) -> Result<Vec<Channel>, CoreError> {
        self.repo.list_by_organization(org).await
    }

    pub async fn create_thread(&mut self, cmd: CreateThreadCommand) -> Result<Channel, CoreError> {
        if cmd.name.trim().is_empty() {
            return Err(CoreError::Conflict(
                "thread name cannot be blank".to_owned(),
            ));
        }
        let parent = self
            .repo
            .find_by_id(cmd.parent_id)
            .await?
            .ok_or(CoreError::NotFound)?;
        if parent.channel_type != ChannelType::Text {
            return Err(CoreError::Conflict(
                "thread parent must be a TEXT channel".to_owned(),
            ));
        }
        if parent.organization_id != cmd.organization_id {
            return Err(CoreError::Conflict(
                "thread parent must belong to the same organization".to_owned(),
            ));
        }
        let now = Utc::now();
        let thread = Channel {
            id: ChannelId(generate_uuid_v7()),
            organization_id: cmd.organization_id,
            channel_type: ChannelType::Thread,
            name: cmd.name,
            topic: None,
            position: 0,
            category_id: None,
            parent_id: Some(cmd.parent_id),
            origin_message_id: cmd.origin_message_id,
            archived: false,
            created_at: now,
            updated_at: now,
        };
        let saved = self.repo.insert(&thread).await?;
        self.events
            .publish(DomainEvent::ThreadCreated(saved.clone()))
            .await?;
        Ok(saved)
    }

    pub async fn update_thread(&mut self, cmd: UpdateThreadCommand) -> Result<Channel, CoreError> {
        if cmd.name.trim().is_empty() {
            return Err(CoreError::Conflict(
                "thread name cannot be blank".to_owned(),
            ));
        }
        let existing = self
            .repo
            .find_by_id(cmd.id)
            .await?
            .ok_or(CoreError::NotFound)?;
        let updated = Channel {
            name: cmd.name,
            archived: cmd.archived,
            updated_at: Utc::now(),
            ..existing
        };
        let saved = self.repo.update(&updated).await?;
        self.events
            .publish(DomainEvent::ThreadUpdated(saved.clone()))
            .await?;
        Ok(saved)
    }

    pub async fn delete_thread(&mut self, id: ChannelId) -> Result<(), CoreError> {
        let existing = self.repo.find_by_id(id).await?.ok_or(CoreError::NotFound)?;
        self.repo.delete(id).await?;
        self.events
            .publish(DomainEvent::ThreadDeleted {
                organization_id: existing.organization_id,
                channel_id: id,
            })
            .await?;
        Ok(())
    }

    pub async fn list_threads(&mut self, parent: ChannelId) -> Result<Vec<Channel>, CoreError> {
        self.repo.list_threads(parent).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        ChannelId, ChannelType,
        domain::channel::ports::MockChannelRepository,
        events::{DomainEvent, MockEventPublisher},
    };
    use common::OrganizationId;
    use mockall::predicate::eq;
    use uuid::Uuid;

    fn text_channel(id: ChannelId, org: OrganizationId) -> Channel {
        use chrono::Utc;
        Channel {
            id,
            organization_id: org,
            channel_type: ChannelType::Text,
            name: "general".to_owned(),
            topic: None,
            position: 0,
            category_id: None,
            parent_id: None,
            origin_message_id: None,
            archived: false,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    #[tokio::test]
    async fn create_channel_publishes_channel_created() {
        let org = OrganizationId(Uuid::new_v4());
        let mut repo = MockChannelRepository::new();
        repo.expect_insert().times(1).returning(|c| {
            let c = c.clone();
            Box::pin(async move { Ok(c) })
        });

        let mut events = MockEventPublisher::new();
        events
            .expect_publish()
            .withf(|e| matches!(e, DomainEvent::ChannelCreated(_)))
            .times(1)
            .returning(|_| Box::pin(async { Ok(()) }));

        let mut svc = ChannelService::new(repo, events);
        let result = svc
            .create_channel(CreateChannelCommand {
                organization_id: org,
                category_id: None,
                name: "general".to_owned(),
                topic: None,
                position: 0,
            })
            .await
            .unwrap();

        assert_eq!(result.channel_type, ChannelType::Text);
    }

    #[tokio::test]
    async fn create_thread_requires_text_parent_in_same_org() {
        let org = OrganizationId(Uuid::new_v4());
        let parent_id = ChannelId(Uuid::new_v4());

        // Parent is a THREAD (wrong type) — should be rejected
        let mut repo = MockChannelRepository::new();
        repo.expect_find_by_id()
            .with(eq(parent_id))
            .returning(move |_| {
                use chrono::Utc;
                let ch = Channel {
                    id: parent_id,
                    organization_id: org,
                    channel_type: ChannelType::Thread, // wrong!
                    name: "existing-thread".to_owned(),
                    topic: None,
                    position: 0,
                    category_id: None,
                    parent_id: None,
                    origin_message_id: None,
                    archived: false,
                    created_at: Utc::now(),
                    updated_at: Utc::now(),
                };
                Box::pin(async move { Ok(Some(ch)) })
            });

        let events = MockEventPublisher::new();
        let mut svc = ChannelService::new(repo, events);

        let result = svc
            .create_thread(CreateThreadCommand {
                organization_id: org,
                parent_id,
                origin_message_id: None,
                name: "my-thread".to_owned(),
            })
            .await;

        assert!(matches!(result, Err(common::CoreError::Conflict(_))));
    }

    #[tokio::test]
    async fn create_thread_rejects_parent_in_different_org() {
        let org = OrganizationId(Uuid::new_v4());
        let other_org = OrganizationId(Uuid::new_v4());
        let parent_id = ChannelId(Uuid::new_v4());

        let mut repo = MockChannelRepository::new();
        repo.expect_find_by_id()
            .with(eq(parent_id))
            .returning(move |_| {
                use chrono::Utc;
                let ch = Channel {
                    id: parent_id,
                    organization_id: other_org, // wrong org!
                    channel_type: ChannelType::Text,
                    name: "general".to_owned(),
                    topic: None,
                    position: 0,
                    category_id: None,
                    parent_id: None,
                    origin_message_id: None,
                    archived: false,
                    created_at: Utc::now(),
                    updated_at: Utc::now(),
                };
                Box::pin(async move { Ok(Some(ch)) })
            });

        let events = MockEventPublisher::new();
        let mut svc = ChannelService::new(repo, events);

        let result = svc
            .create_thread(CreateThreadCommand {
                organization_id: org,
                parent_id,
                origin_message_id: None,
                name: "my-thread".to_owned(),
            })
            .await;

        assert!(matches!(result, Err(common::CoreError::Conflict(_))));
    }

    #[tokio::test]
    async fn create_thread_happy_path_publishes_thread_created() {
        let org = OrganizationId(Uuid::new_v4());
        let parent_id = ChannelId(Uuid::new_v4());

        let mut repo = MockChannelRepository::new();
        repo.expect_find_by_id()
            .with(eq(parent_id))
            .returning(move |_| Box::pin(async move { Ok(Some(text_channel(parent_id, org))) }));
        repo.expect_insert().times(1).returning(|c| {
            let c = c.clone();
            Box::pin(async move { Ok(c) })
        });

        let mut events = MockEventPublisher::new();
        events
            .expect_publish()
            .withf(|e| matches!(e, DomainEvent::ThreadCreated(_)))
            .times(1)
            .returning(|_| Box::pin(async { Ok(()) }));

        let mut svc = ChannelService::new(repo, events);
        let result = svc
            .create_thread(CreateThreadCommand {
                organization_id: org,
                parent_id,
                origin_message_id: None,
                name: "my-thread".to_owned(),
            })
            .await
            .unwrap();

        assert_eq!(result.channel_type, ChannelType::Thread);
        assert_eq!(result.parent_id, Some(parent_id));
    }

    #[tokio::test]
    async fn delete_channel_publishes_channel_deleted() {
        let org = OrganizationId(Uuid::new_v4());
        let id = ChannelId(Uuid::new_v4());

        let mut repo = MockChannelRepository::new();
        repo.expect_find_by_id()
            .with(eq(id))
            .returning(move |_| Box::pin(async move { Ok(Some(text_channel(id, org))) }));
        repo.expect_delete()
            .with(eq(id))
            .returning(|_| Box::pin(async { Ok(()) }));

        let mut events = MockEventPublisher::new();
        events
            .expect_publish()
            .withf(|e| matches!(e, DomainEvent::ChannelDeleted { .. }))
            .times(1)
            .returning(|_| Box::pin(async { Ok(()) }));

        let mut svc = ChannelService::new(repo, events);
        svc.delete_channel(id).await.unwrap();
    }

    #[tokio::test]
    async fn update_channel_publishes_channel_updated() {
        let org = OrganizationId(Uuid::new_v4());
        let id = ChannelId(Uuid::new_v4());

        let mut repo = MockChannelRepository::new();
        repo.expect_find_by_id()
            .with(eq(id))
            .times(1)
            .returning(move |_| Box::pin(async move { Ok(Some(text_channel(id, org))) }));
        repo.expect_update().times(1).returning(|c| {
            let c = c.clone();
            Box::pin(async move { Ok(c) })
        });

        let mut events = MockEventPublisher::new();
        events
            .expect_publish()
            .withf(|e| matches!(e, DomainEvent::ChannelUpdated(_)))
            .times(1)
            .returning(|_| Box::pin(async { Ok(()) }));

        let mut svc = ChannelService::new(repo, events);
        let result = svc
            .update_channel(UpdateChannelCommand {
                id,
                category_id: None,
                name: "updated-general".to_owned(),
                topic: None,
                position: 1,
            })
            .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn update_thread_publishes_thread_updated() {
        let org = OrganizationId(Uuid::new_v4());
        let id = ChannelId(Uuid::new_v4());

        let mut repo = MockChannelRepository::new();
        repo.expect_find_by_id()
            .with(eq(id))
            .times(1)
            .returning(move |_| Box::pin(async move { Ok(Some(text_channel(id, org))) }));
        repo.expect_update().times(1).returning(|c| {
            let c = c.clone();
            Box::pin(async move { Ok(c) })
        });

        let mut events = MockEventPublisher::new();
        events
            .expect_publish()
            .withf(|e| matches!(e, DomainEvent::ThreadUpdated(_)))
            .times(1)
            .returning(|_| Box::pin(async { Ok(()) }));

        let mut svc = ChannelService::new(repo, events);
        let result = svc
            .update_thread(UpdateThreadCommand {
                id,
                name: "updated-thread".to_owned(),
                archived: false,
            })
            .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn delete_thread_publishes_thread_deleted() {
        let org = OrganizationId(Uuid::new_v4());
        let id = ChannelId(Uuid::new_v4());

        let mut repo = MockChannelRepository::new();
        repo.expect_find_by_id()
            .with(eq(id))
            .times(1)
            .returning(move |_| Box::pin(async move { Ok(Some(text_channel(id, org))) }));
        repo.expect_delete()
            .with(eq(id))
            .times(1)
            .returning(|_| Box::pin(async { Ok(()) }));

        let mut events = MockEventPublisher::new();
        events
            .expect_publish()
            .withf(|e| matches!(e, DomainEvent::ThreadDeleted { .. }))
            .times(1)
            .returning(|_| Box::pin(async { Ok(()) }));

        let mut svc = ChannelService::new(repo, events);
        svc.delete_thread(id).await.unwrap();
    }

    #[tokio::test]
    async fn create_thread_rejects_missing_parent() {
        let org = OrganizationId(Uuid::new_v4());
        let parent_id = ChannelId(Uuid::new_v4());

        let mut repo = MockChannelRepository::new();
        repo.expect_find_by_id()
            .with(eq(parent_id))
            .times(1)
            .returning(|_| Box::pin(async { Ok(None) }));

        let events = MockEventPublisher::new();
        let mut svc = ChannelService::new(repo, events);

        let result = svc
            .create_thread(CreateThreadCommand {
                organization_id: org,
                parent_id,
                origin_message_id: None,
                name: "orphan-thread".to_owned(),
            })
            .await;

        assert!(matches!(result, Err(common::CoreError::NotFound)));
    }
}
