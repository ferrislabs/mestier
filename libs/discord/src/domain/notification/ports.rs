use std::future::Future;

use common::{CoreError, OrganizationId, UserId};

use crate::{NotificationId, domain::Notification};

use super::commands::{CreateNotification, MarkNotificationRead};

#[cfg_attr(test, mockall::automock)]
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
