use chrono::Utc;
use common::CoreError;

use crate::{
    OrganizationId, Presence, UserId,
    domain::presence::{
        commands::{SetPresenceCommand, StartTypingCommand},
        ports::PresenceRepository,
    },
    events::{DomainEvent, EventPublisher},
};

pub struct PresenceService<R, E> {
    repo: R,
    events: E,
}

impl<R, E> PresenceService<R, E>
where
    R: PresenceRepository,
    E: EventPublisher,
{
    pub fn new(repo: R, events: E) -> Self {
        Self { repo, events }
    }

    pub async fn set_presence(&mut self, cmd: SetPresenceCommand) -> Result<Presence, CoreError> {
        let presence = Presence {
            organization_id: cmd.organization_id,
            user_id: cmd.user_id,
            status: cmd.status,
            updated_at: Utc::now(),
        };
        let saved = self.repo.upsert(&presence).await?;
        self.events
            .publish(DomainEvent::PresenceUpdated {
                organization_id: cmd.organization_id,
                user_id: cmd.user_id,
                status: cmd.status,
            })
            .await?;
        Ok(saved)
    }

    pub async fn get_presence(
        &mut self,
        org: OrganizationId,
        user: UserId,
    ) -> Result<Option<Presence>, CoreError> {
        self.repo.find(org, user).await
    }

    pub async fn list_presence(&mut self, org: OrganizationId) -> Result<Vec<Presence>, CoreError> {
        self.repo.list_by_organization(org).await
    }

    /// Publishes `TypingStarted` with NO database write — ephemeral signal only.
    pub async fn start_typing(&mut self, cmd: StartTypingCommand) -> Result<(), CoreError> {
        self.events
            .publish(DomainEvent::TypingStarted {
                organization_id: cmd.organization_id,
                channel_id: cmd.channel_id,
                user_id: cmd.user_id,
            })
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        ChannelId, PresenceStatus,
        domain::presence::ports::MockPresenceRepository,
        events::{DomainEvent, MockEventPublisher},
    };
    use common::OrganizationId;
    use uuid::Uuid;

    fn make_presence(org: OrganizationId, user: UserId) -> Presence {
        use chrono::Utc;
        Presence {
            organization_id: org,
            user_id: user,
            status: PresenceStatus::Online,
            updated_at: Utc::now(),
        }
    }

    #[tokio::test]
    async fn set_presence_upserts_and_publishes_presence_updated() {
        let org = OrganizationId(Uuid::new_v4());
        let user = UserId(Uuid::new_v4());

        let mut repo = MockPresenceRepository::new();
        repo.expect_upsert().times(1).returning(move |p| {
            let p = p.clone();
            Box::pin(async move { Ok(p) })
        });

        let mut events = MockEventPublisher::new();
        events
            .expect_publish()
            .withf(|e| matches!(e, DomainEvent::PresenceUpdated { .. }))
            .times(1)
            .returning(|_| Box::pin(async { Ok(()) }));

        let mut svc = PresenceService::new(repo, events);
        let result = svc
            .set_presence(SetPresenceCommand {
                organization_id: org,
                user_id: user,
                status: PresenceStatus::Dnd,
            })
            .await
            .unwrap();

        assert_eq!(result.status, PresenceStatus::Dnd);
    }

    #[tokio::test]
    async fn start_typing_publishes_typing_started_without_db_write() {
        let org = OrganizationId(Uuid::new_v4());
        let user = UserId(Uuid::new_v4());
        let channel_id = ChannelId(Uuid::new_v4());

        // Repo must NOT be called at all
        let mut repo = MockPresenceRepository::new();
        repo.expect_upsert().times(0);
        repo.expect_find().times(0);
        repo.expect_list_by_organization().times(0);

        let mut events = MockEventPublisher::new();
        events
            .expect_publish()
            .withf(|e| matches!(e, DomainEvent::TypingStarted { .. }))
            .times(1)
            .returning(|_| Box::pin(async { Ok(()) }));

        let mut svc = PresenceService::new(repo, events);
        svc.start_typing(StartTypingCommand {
            organization_id: org,
            channel_id,
            user_id: user,
        })
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn get_presence_returns_none_when_not_set() {
        let org = OrganizationId(Uuid::new_v4());
        let user = UserId(Uuid::new_v4());

        let mut repo = MockPresenceRepository::new();
        repo.expect_find()
            .with(mockall::predicate::eq(org), mockall::predicate::eq(user))
            .returning(|_, _| Box::pin(async { Ok(None) }));

        let events = MockEventPublisher::new();
        let mut svc = PresenceService::new(repo, events);

        let result = svc.get_presence(org, user).await.unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn list_presence_delegates_to_repo() {
        let org = OrganizationId(Uuid::new_v4());
        let user = UserId(Uuid::new_v4());

        let mut repo = MockPresenceRepository::new();
        repo.expect_list_by_organization()
            .with(mockall::predicate::eq(org))
            .returning(move |_| Box::pin(async move { Ok(vec![make_presence(org, user)]) }));

        let events = MockEventPublisher::new();
        let mut svc = PresenceService::new(repo, events);

        let list = svc.list_presence(org).await.unwrap();
        assert_eq!(list.len(), 1);
    }
}
