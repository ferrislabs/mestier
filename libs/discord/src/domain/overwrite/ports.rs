use std::future::Future;

use common::CoreError;

use crate::ChannelId;
use crate::domain::{ChannelPermissionOverwrite, OverwriteTarget};

use super::commands::UpsertChannelOverwrite;

#[cfg_attr(any(test, feature = "mock"), mockall::automock)]
pub trait OverwriteRepository: Send + Sync {
    fn upsert(
        &self,
        command: UpsertChannelOverwrite,
    ) -> impl Future<Output = Result<ChannelPermissionOverwrite, CoreError>> + Send;
    fn delete(
        &self,
        channel_id: ChannelId,
        target: OverwriteTarget,
    ) -> impl Future<Output = Result<(), CoreError>> + Send;
    fn list_for_channel(
        &self,
        channel_id: ChannelId,
    ) -> impl Future<Output = Result<Vec<ChannelPermissionOverwrite>, CoreError>> + Send;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{ChannelPermissionOverwrite, OverwriteTarget};
    use crate::{ChannelId, OverwriteId};
    use chrono::Utc;
    use common::OrganizationId;
    use uuid::Uuid;

    fn make_overwrite(channel_id: ChannelId) -> ChannelPermissionOverwrite {
        ChannelPermissionOverwrite {
            id: OverwriteId(Uuid::new_v4()),
            channel_id,
            organization_id: OrganizationId(Uuid::new_v4()),
            target: OverwriteTarget::Everyone,
            allow: 0,
            deny: 0,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    #[tokio::test]
    async fn mock_overwrite_repository_upsert_returns_overwrite() {
        let mut repo = MockOverwriteRepository::new();
        let channel_id = ChannelId(Uuid::new_v4());
        let org_id = OrganizationId(Uuid::new_v4());
        let overwrite = make_overwrite(channel_id);
        let overwrite_clone = overwrite.clone();

        repo.expect_upsert().times(1).returning(move |_| {
            let o = overwrite_clone.clone();
            Box::pin(async move { Ok(o) })
        });

        let cmd = super::UpsertChannelOverwrite {
            channel_id,
            organization_id: org_id,
            target: OverwriteTarget::Everyone,
            allow: 0,
            deny: 0,
        };

        let result = repo.upsert(cmd).await.unwrap();
        assert_eq!(result.channel_id, channel_id);
        assert_eq!(result.target, OverwriteTarget::Everyone);
    }

    #[tokio::test]
    async fn mock_overwrite_repository_delete_returns_ok() {
        let mut repo = MockOverwriteRepository::new();
        let channel_id = ChannelId(Uuid::new_v4());

        repo.expect_delete()
            .times(1)
            .returning(|_, _| Box::pin(async { Ok(()) }));

        repo.delete(channel_id, OverwriteTarget::Everyone)
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn mock_overwrite_repository_list_for_channel_returns_vec() {
        let mut repo = MockOverwriteRepository::new();
        let channel_id = ChannelId(Uuid::new_v4());
        let overwrite = make_overwrite(channel_id);
        let overwrite_clone = overwrite.clone();

        repo.expect_list_for_channel().times(1).returning(move |_| {
            let o = overwrite_clone.clone();
            Box::pin(async move { Ok(vec![o]) })
        });

        let result = repo.list_for_channel(channel_id).await.unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].channel_id, channel_id);
    }
}
