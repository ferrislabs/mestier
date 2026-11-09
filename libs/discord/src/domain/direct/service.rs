use chrono::Utc;
use common::{CoreError, OrganizationId, UserId, generate_uuid_v7};

use crate::{Channel, ChannelId, ChannelRepository, ChannelType};

use super::{DirectChannel, DirectChannelRepository, DirectPair};

/// Direct messages are private: opening one deliberately publishes no event.
/// The gateway fans organisation-level events out to everyone, and a
/// conversation's existence must not be among them.
pub struct DirectChannelService<C, D> {
    channels: C,
    directs: D,
}

impl<C, D> DirectChannelService<C, D>
where
    C: ChannelRepository,
    D: DirectChannelRepository,
{
    pub fn new(channels: C, directs: D) -> Self {
        Self { channels, directs }
    }

    /// The conversation between `caller` and `peer`, created on first use.
    pub async fn open(
        &mut self,
        organization_id: OrganizationId,
        caller: UserId,
        peer: UserId,
    ) -> Result<DirectChannel, CoreError> {
        let pair = DirectPair::new(caller, peer)?;
        if let Some(existing) = self.directs.find_by_pair(organization_id, pair).await? {
            return Ok(existing);
        }

        let now = Utc::now();
        let channel = Channel {
            id: ChannelId(generate_uuid_v7()),
            organization_id,
            channel_type: ChannelType::Direct,
            name: "direct".to_owned(),
            topic: None,
            position: 0,
            category_id: None,
            parent_id: None,
            origin_message_id: None,
            archived: false,
            project_id: None,
            created_at: now,
            updated_at: now,
        };
        let channel = self.channels.insert(&channel).await?;
        self.directs.insert(&channel, pair).await?;
        Ok(DirectChannel { channel, pair })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::channel::ports::MockChannelRepository;
    use crate::domain::direct::ports::MockDirectChannelRepository;
    use uuid::Uuid;

    fn user(n: u128) -> UserId {
        UserId(Uuid::from_u128(n))
    }

    fn org() -> OrganizationId {
        OrganizationId(Uuid::from_u128(99))
    }

    fn direct_channel(pair: DirectPair) -> DirectChannel {
        let now = Utc::now();
        DirectChannel {
            channel: Channel {
                id: ChannelId(Uuid::from_u128(7)),
                organization_id: org(),
                channel_type: ChannelType::Direct,
                name: "direct".to_owned(),
                topic: None,
                position: 0,
                category_id: None,
                parent_id: None,
                origin_message_id: None,
                archived: false,
                project_id: None,
                created_at: now,
                updated_at: now,
            },
            pair,
        }
    }

    #[tokio::test]
    async fn open_returns_the_existing_conversation_without_creating_one() {
        let pair = DirectPair::new(user(1), user(2)).unwrap();
        let existing = direct_channel(pair);
        let expected_id = existing.channel.id;

        let mut directs = MockDirectChannelRepository::new();
        directs
            .expect_find_by_pair()
            .times(1)
            .returning(move |_, _| {
                let found = existing.clone();
                Box::pin(async move { Ok(Some(found)) })
            });
        directs.expect_insert().never();
        let mut channels = MockChannelRepository::new();
        channels.expect_insert().never();

        let mut service = DirectChannelService::new(channels, directs);
        let opened = service.open(org(), user(2), user(1)).await.unwrap();

        assert_eq!(opened.channel.id, expected_id);
    }

    #[tokio::test]
    async fn open_creates_a_direct_channel_for_a_new_pair() {
        let mut directs = MockDirectChannelRepository::new();
        directs
            .expect_find_by_pair()
            .times(1)
            .returning(|_, _| Box::pin(async { Ok(None) }));
        directs
            .expect_insert()
            .times(1)
            .withf(|channel, pair| {
                channel.channel_type == ChannelType::Direct
                    && channel.category_id.is_none()
                    && pair.includes(UserId(Uuid::from_u128(1)))
            })
            .returning(|_, _| Box::pin(async { Ok(()) }));
        let mut channels = MockChannelRepository::new();
        channels.expect_insert().times(1).returning(|channel| {
            let saved = channel.clone();
            Box::pin(async move { Ok(saved) })
        });

        let mut service = DirectChannelService::new(channels, directs);
        let opened = service.open(org(), user(1), user(2)).await.unwrap();

        assert_eq!(opened.channel.channel_type, ChannelType::Direct);
        assert_eq!(opened.pair.peer_of(user(1)), Some(user(2)));
    }

    #[tokio::test]
    async fn open_refuses_a_conversation_with_oneself() {
        let mut service = DirectChannelService::new(
            MockChannelRepository::new(),
            MockDirectChannelRepository::new(),
        );

        assert!(matches!(
            service.open(org(), user(1), user(1)).await,
            Err(CoreError::Conflict(_))
        ));
    }
}
