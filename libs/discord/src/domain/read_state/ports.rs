use std::future::Future;

use common::{CoreError, OrganizationId, UserId};

use crate::{ChannelId, domain::ChannelReadState};

use super::commands::MarkChannelReadCommand;

#[cfg_attr(test, mockall::automock)]
pub trait ReadStateRepository: Send + Sync {
    fn upsert(
        &self,
        command: MarkChannelReadCommand,
    ) -> impl Future<Output = Result<Option<ChannelReadState>, CoreError>> + Send;
    fn get(
        &self,
        user_id: UserId,
        channel_id: ChannelId,
    ) -> impl Future<Output = Result<Option<ChannelReadState>, CoreError>> + Send;
    fn unread_channels(
        &self,
        user_id: UserId,
        organization_id: OrganizationId,
    ) -> impl Future<Output = Result<Vec<ChannelId>, CoreError>> + Send;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::ChannelReadState;
    use chrono::Utc;
    use common::OrganizationId;
    use uuid::Uuid;

    #[tokio::test]
    async fn mock_read_state_repository_upsert_returns_some() {
        let mut repo = MockReadStateRepository::new();
        let user_id = UserId(Uuid::new_v4());
        let channel_id = ChannelId(Uuid::new_v4());
        let org_id = OrganizationId(Uuid::new_v4());
        let message_id = crate::MessageId(Uuid::new_v4());

        let state = ChannelReadState {
            organization_id: org_id,
            channel_id,
            user_id,
            last_read_message_id: Some(message_id),
            updated_at: Utc::now(),
        };
        let state_clone = state.clone();

        repo.expect_upsert().times(1).returning(move |_| {
            let s = state_clone.clone();
            Box::pin(async move { Ok(Some(s)) })
        });

        let cmd = super::super::commands::MarkChannelReadCommand {
            organization_id: org_id,
            channel_id,
            user_id,
            message_id,
        };

        let result = repo.upsert(cmd).await.unwrap();
        assert!(result.is_some());
        let returned = result.unwrap();
        assert_eq!(returned.user_id, user_id);
        assert_eq!(returned.channel_id, channel_id);
    }

    #[tokio::test]
    async fn mock_read_state_repository_upsert_returns_none_for_no_op() {
        let mut repo = MockReadStateRepository::new();
        let user_id = UserId(Uuid::new_v4());
        let channel_id = ChannelId(Uuid::new_v4());
        let org_id = OrganizationId(Uuid::new_v4());
        let message_id = crate::MessageId(Uuid::new_v4());

        repo.expect_upsert()
            .times(1)
            .returning(|_| Box::pin(async { Ok(None) }));

        let cmd = super::super::commands::MarkChannelReadCommand {
            organization_id: org_id,
            channel_id,
            user_id,
            message_id,
        };

        let result = repo.upsert(cmd).await.unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn mock_read_state_repository_unread_channels_returns_vec() {
        let mut repo = MockReadStateRepository::new();
        let user_id = UserId(Uuid::new_v4());
        let org_id = OrganizationId(Uuid::new_v4());
        let channel_id = ChannelId(Uuid::new_v4());

        repo.expect_unread_channels()
            .times(1)
            .returning(move |_, _| Box::pin(async move { Ok(vec![channel_id]) }));

        let result = repo.unread_channels(user_id, org_id).await.unwrap();
        assert_eq!(result.len(), 1);
    }
}
