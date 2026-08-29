use chrono::Utc;
use common::{CoreError, ProjectId, generate_uuid_v7};

use crate::{
    Channel, ChannelId, ChannelType, OrganizationId,
    domain::channel::{
        commands::{
            CreateChannelCommand, CreateProjectChannelCommand, CreateThreadCommand,
            UpdateChannelCommand, UpdateThreadCommand,
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
            project_id: None,
            created_at: now,
            updated_at: now,
        };
        let saved = self.repo.insert(&ch).await?;
        self.events
            .publish(DomainEvent::ChannelCreated(saved.clone()))
            .await?;
        Ok(saved)
    }

    /// Creates the one channel a project may have — see
    /// `CreateProjectChannelCommand`'s own doc comment. `uq_channels_project_id`
    /// is the real enforcement (a second write path cannot bypass it); the
    /// check here exists only to answer with a clean `Conflict` instead of a
    /// raw constraint-name error.
    pub async fn create_project_channel(
        &mut self,
        cmd: CreateProjectChannelCommand,
    ) -> Result<Channel, CoreError> {
        if cmd.name.trim().is_empty() {
            return Err(CoreError::Conflict(
                "channel name cannot be blank".to_owned(),
            ));
        }
        if self
            .repo
            .find_by_project_id(cmd.project_id)
            .await?
            .is_some()
        {
            return Err(CoreError::Conflict(
                "this project already has a channel — a project may have at most one".to_owned(),
            ));
        }

        let now = Utc::now();
        let ch = Channel {
            id: ChannelId(generate_uuid_v7()),
            organization_id: cmd.organization_id,
            channel_type: ChannelType::Text,
            name: cmd.name,
            topic: None,
            position: 0,
            category_id: None,
            parent_id: None,
            origin_message_id: None,
            archived: false,
            project_id: Some(cmd.project_id),
            created_at: now,
            updated_at: now,
        };
        let saved = self.repo.insert(&ch).await?;
        self.events
            .publish(DomainEvent::ChannelCreated(saved.clone()))
            .await?;
        Ok(saved)
    }

    /// Cascades a project's own archive/restore into its channel. Called by
    /// `MestierUseCase::archive_project`/`restore_project`, in the same
    /// transaction as the project's own `archived_at` write — the
    /// conversation about a job follows the job's own lifecycle rather than
    /// being archived by hand, and stays fully readable either way (archiving
    /// is not deleting).
    ///
    /// Returns `None` when the project has no channel, which is the common
    /// case: most projects (internal admin, a quick job) never grow one.
    pub async fn set_project_channel_archived(
        &mut self,
        project_id: ProjectId,
        archived: bool,
    ) -> Result<Option<Channel>, CoreError> {
        let Some(channel) = self.repo.find_by_project_id(project_id).await? else {
            return Ok(None);
        };
        let saved = self.repo.set_archived(channel.id, archived).await?;
        self.events
            .publish(DomainEvent::ChannelUpdated(saved.clone()))
            .await?;
        Ok(Some(saved))
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
            // Immutable via this generic update, same as `channel_type` and
            // `parent_id` above: nothing moves a channel from one project to
            // another, or attaches one after the fact. See `Channel::project_id`.
            project_id: existing.project_id,
            created_at: existing.created_at,
            updated_at: Utc::now(),
        };
        let saved = self.repo.update(&updated).await?;
        self.events
            .publish(DomainEvent::ChannelUpdated(saved.clone()))
            .await?;
        Ok(saved)
    }

    /// Refuses to hard-delete a project's channel. This codebase never
    /// hard-deletes a project either (`DELETE /projects/{id}` archives — see
    /// `handlers-planning`'s `archive.rs`), so the only way a project channel
    /// is ever taken out of the running conversation list is by archiving the
    /// project it belongs to (`set_project_channel_archived`), which keeps
    /// the messages readable. Letting the generic delete-channel endpoint
    /// remove it regardless would silently destroy that history through a
    /// second path — exactly what this issue rules out.
    pub async fn delete_channel(&mut self, id: ChannelId) -> Result<(), CoreError> {
        let existing = self.repo.find_by_id(id).await?.ok_or(CoreError::NotFound)?;
        if existing.project_id.is_some() {
            return Err(CoreError::Conflict(
                "a project channel cannot be deleted directly; archive its project instead"
                    .to_owned(),
            ));
        }
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
            // A thread never carries a project of its own —
            // `chk_channels_thread_no_project` forbids it at the database too.
            project_id: None,
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

    fn thread_channel(id: ChannelId, org: OrganizationId) -> Channel {
        Channel {
            channel_type: ChannelType::Thread,
            parent_id: Some(ChannelId(Uuid::new_v4())),
            ..text_channel(id, org)
        }
    }

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
            project_id: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    fn project_channel(id: ChannelId, org: OrganizationId, project_id: ProjectId) -> Channel {
        Channel {
            project_id: Some(project_id),
            ..text_channel(id, org)
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
                    project_id: None,
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
                    project_id: None,
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
            .returning(move |_| Box::pin(async move { Ok(Some(thread_channel(id, org))) }));
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
            .returning(move |_| Box::pin(async move { Ok(Some(thread_channel(id, org))) }));
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

    // -----------------------------------------------------------------
    // #345 — a project's one channel
    // -----------------------------------------------------------------

    #[tokio::test]
    async fn create_project_channel_sets_project_id_and_publishes_channel_created() {
        let org = OrganizationId(Uuid::new_v4());
        let project_id = ProjectId(Uuid::new_v4());

        let mut repo = MockChannelRepository::new();
        repo.expect_find_by_project_id()
            .with(eq(project_id))
            .times(1)
            .returning(|_| Box::pin(async { Ok(None) }));
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
            .create_project_channel(CreateProjectChannelCommand {
                organization_id: org,
                project_id,
                name: "Toiture Dupont".to_owned(),
            })
            .await
            .unwrap();

        assert_eq!(result.channel_type, ChannelType::Text);
        assert_eq!(result.project_id, Some(project_id));
    }

    #[tokio::test]
    async fn create_project_channel_refuses_a_second_channel_for_the_same_project() {
        let org = OrganizationId(Uuid::new_v4());
        let project_id = ProjectId(Uuid::new_v4());
        let existing_id = ChannelId(Uuid::new_v4());

        let mut repo = MockChannelRepository::new();
        repo.expect_find_by_project_id()
            .with(eq(project_id))
            .times(1)
            .returning(move |_| {
                Box::pin(async move { Ok(Some(project_channel(existing_id, org, project_id))) })
            });

        let events = MockEventPublisher::new();
        let mut svc = ChannelService::new(repo, events);

        let result = svc
            .create_project_channel(CreateProjectChannelCommand {
                organization_id: org,
                project_id,
                name: "Second channel".to_owned(),
            })
            .await;

        assert!(matches!(result, Err(common::CoreError::Conflict(_))));
    }

    #[tokio::test]
    async fn delete_channel_refuses_a_project_channel() {
        let org = OrganizationId(Uuid::new_v4());
        let project_id = ProjectId(Uuid::new_v4());
        let id = ChannelId(Uuid::new_v4());

        let mut repo = MockChannelRepository::new();
        repo.expect_find_by_id()
            .with(eq(id))
            .times(1)
            .returning(move |_| {
                Box::pin(async move { Ok(Some(project_channel(id, org, project_id))) })
            });
        // No `expect_delete`: reaching the repository's `delete` would be the
        // bug this test guards against.

        let events = MockEventPublisher::new();
        let mut svc = ChannelService::new(repo, events);

        let result = svc.delete_channel(id).await;

        assert!(matches!(result, Err(common::CoreError::Conflict(_))));
    }

    #[tokio::test]
    async fn set_project_channel_archived_cascades_to_the_channel() {
        let org = OrganizationId(Uuid::new_v4());
        let project_id = ProjectId(Uuid::new_v4());
        let channel_id = ChannelId(Uuid::new_v4());

        let mut repo = MockChannelRepository::new();
        repo.expect_find_by_project_id()
            .with(eq(project_id))
            .times(1)
            .returning(move |_| {
                Box::pin(async move { Ok(Some(project_channel(channel_id, org, project_id))) })
            });
        repo.expect_set_archived()
            .with(eq(channel_id), eq(true))
            .times(1)
            .returning(move |id, archived| {
                let mut ch = project_channel(id, org, project_id);
                ch.archived = archived;
                Box::pin(async move { Ok(ch) })
            });

        let mut events = MockEventPublisher::new();
        events
            .expect_publish()
            .withf(|e| matches!(e, DomainEvent::ChannelUpdated(_)))
            .times(1)
            .returning(|_| Box::pin(async { Ok(()) }));

        let mut svc = ChannelService::new(repo, events);
        let result = svc
            .set_project_channel_archived(project_id, true)
            .await
            .unwrap();

        assert!(result.unwrap().archived);
    }

    #[tokio::test]
    async fn set_project_channel_archived_is_a_no_op_when_the_project_has_no_channel() {
        let project_id = ProjectId(Uuid::new_v4());

        let mut repo = MockChannelRepository::new();
        repo.expect_find_by_project_id()
            .with(eq(project_id))
            .times(1)
            .returning(|_| Box::pin(async { Ok(None) }));
        // No `expect_set_archived` and no `expect_publish`: a project with no
        // channel must not touch the repository or emit anything further.

        let events = MockEventPublisher::new();
        let mut svc = ChannelService::new(repo, events);

        let result = svc
            .set_project_channel_archived(project_id, true)
            .await
            .unwrap();

        assert!(result.is_none());
    }
}
