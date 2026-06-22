use common::CoreError;
use discord::{
    MarkNotificationRead, Notification, NotificationId, NotificationRepository, OrganizationId,
    UserId,
};
use mestier_macros::transactional;

use crate::application::MestierUseCase;

impl MestierUseCase {
    #[transactional(notification)]
    pub async fn list_notifications(
        &self,
        user_id: UserId,
        organization_id: OrganizationId,
        unread_only: bool,
        before: Option<NotificationId>,
        limit: i64,
    ) -> Result<Vec<Notification>, CoreError> {
        notification_repository
            .list(user_id, organization_id, unread_only, before, limit)
            .await
    }

    #[transactional(notification)]
    pub async fn mark_notification_read(
        &self,
        command: MarkNotificationRead,
    ) -> Result<(), CoreError> {
        notification_repository
            .mark_read(command.notification_id, command.user_id)
            .await
    }

    #[transactional(notification)]
    pub async fn mark_all_notifications_read(
        &self,
        user_id: UserId,
        organization_id: OrganizationId,
    ) -> Result<(), CoreError> {
        notification_repository
            .mark_all_read(user_id, organization_id)
            .await
    }
}
