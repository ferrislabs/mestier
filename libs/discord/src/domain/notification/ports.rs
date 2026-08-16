use std::future::Future;

use common::{CoreError, OrganizationId, UserId};

use crate::{NotificationId, domain::Notification};

use super::commands::CreateNotification;

#[cfg_attr(any(test, feature = "mock"), mockall::automock)]
pub trait NotificationRepository: Send + Sync {
    fn create(
        &self,
        command: CreateNotification,
    ) -> impl Future<Output = Result<Notification, CoreError>> + Send;
    fn list(
        &self,
        user_id: UserId,
        organization_id: OrganizationId,
        unread_only: bool,
        before: Option<NotificationId>,
        limit: i64,
    ) -> impl Future<Output = Result<Vec<Notification>, CoreError>> + Send;
    fn mark_read(
        &self,
        notification_id: NotificationId,
        user_id: UserId,
    ) -> impl Future<Output = Result<(), CoreError>> + Send;
    fn mark_all_read(
        &self,
        user_id: UserId,
        organization_id: OrganizationId,
    ) -> impl Future<Output = Result<(), CoreError>> + Send;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ChannelId, MessageId, NotificationId, NotificationKind, domain::Notification};
    use chrono::Utc;
    use uuid::Uuid;

    fn make_notification(organization_id: OrganizationId, user_id: UserId) -> Notification {
        Notification {
            id: NotificationId(Uuid::new_v4()),
            organization_id,
            user_id,
            channel_id: ChannelId(Uuid::new_v4()),
            message_id: MessageId(Uuid::new_v4()),
            kind: NotificationKind::Mention,
            read_at: None,
            created_at: Utc::now(),
        }
    }

    #[tokio::test]
    async fn mock_notification_repository_create_returns_notification() {
        let mut repo = MockNotificationRepository::new();
        let organization_id = OrganizationId(Uuid::new_v4());
        let user_id = UserId(Uuid::new_v4());
        let channel_id = ChannelId(Uuid::new_v4());
        let message_id = MessageId(Uuid::new_v4());
        let notif = make_notification(organization_id, user_id);
        let notif_clone = notif.clone();

        repo.expect_create().times(1).returning(move |_| {
            let n = notif_clone.clone();
            Box::pin(async move { Ok(n) })
        });

        let cmd = super::super::commands::CreateNotification {
            organization_id,
            user_id,
            channel_id,
            message_id,
            kind: NotificationKind::Mention,
        };

        let result = repo.create(cmd).await.unwrap();
        assert_eq!(result.user_id, user_id);
        assert_eq!(result.organization_id, organization_id);
        assert!(result.read_at.is_none());
    }

    #[tokio::test]
    async fn mock_notification_repository_list_returns_vec() {
        let mut repo = MockNotificationRepository::new();
        let organization_id = OrganizationId(Uuid::new_v4());
        let user_id = UserId(Uuid::new_v4());
        let notif = make_notification(organization_id, user_id);
        let notif_clone = notif.clone();

        repo.expect_list().times(1).returning(move |_, _, _, _, _| {
            let n = notif_clone.clone();
            Box::pin(async move { Ok(vec![n]) })
        });

        let result = repo
            .list(user_id, organization_id, false, None, 50)
            .await
            .unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].user_id, user_id);
    }

    #[tokio::test]
    async fn mock_notification_repository_list_unread_only_returns_empty_when_no_unread() {
        let mut repo = MockNotificationRepository::new();
        let organization_id = OrganizationId(Uuid::new_v4());
        let user_id = UserId(Uuid::new_v4());

        repo.expect_list()
            .times(1)
            .returning(|_, _, _, _, _| Box::pin(async { Ok(vec![]) }));

        let result = repo
            .list(user_id, organization_id, true, None, 50)
            .await
            .unwrap();
        assert!(result.is_empty());
    }

    #[tokio::test]
    async fn mock_notification_repository_mark_read_succeeds() {
        let mut repo = MockNotificationRepository::new();
        let notification_id = NotificationId(Uuid::new_v4());
        let user_id = UserId(Uuid::new_v4());

        repo.expect_mark_read()
            .times(1)
            .returning(|_, _| Box::pin(async { Ok(()) }));

        repo.mark_read(notification_id, user_id).await.unwrap();
    }

    #[tokio::test]
    async fn mock_notification_repository_mark_all_read_succeeds() {
        let mut repo = MockNotificationRepository::new();
        let organization_id = OrganizationId(Uuid::new_v4());
        let user_id = UserId(Uuid::new_v4());

        repo.expect_mark_all_read()
            .times(1)
            .returning(|_, _| Box::pin(async { Ok(()) }));

        repo.mark_all_read(user_id, organization_id).await.unwrap();
    }
}
