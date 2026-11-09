use std::collections::{HashMap, HashSet};

use chrono::{DateTime, Utc};
use common::CoreError;
use discord::{
    Channel, ChannelId, ChannelRepository, ChannelType, DirectChannelRepository,
    MarkNotificationRead, Message, MessageId, MessageRepository, Notification, NotificationId,
    NotificationKind, NotificationLevel, NotificationPreferenceRepository, NotificationRepository,
    OrganizationId, ReadStateRepository, UserId, effective_level,
};
use mestier_macros::transactional;

use crate::application::MestierUseCase;

#[cfg(test)]
mod tests;

/// Why something is in a person's activity.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActivityKind {
    Mention,
    Reply,
    Unread,
}

impl ActivityKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Mention => "MENTION",
            Self::Reply => "REPLY",
            Self::Unread => "UNREAD",
        }
    }
}

/// One line of the activity: a message to read, and why.
#[derive(Debug, Clone)]
pub struct ActivityEntry {
    pub kind: ActivityKind,
    pub channel_id: ChannelId,
    pub message: Message,
    /// The notification behind a mention, so it can be marked read.
    pub notification_id: Option<NotificationId>,
    /// How many messages wait in the channel, for an unread entry.
    pub unread_count: Option<i64>,
    pub read: bool,
    pub created_at: DateTime<Utc>,
}

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

    /// Records how much of `channel` the caller wants to hear about.
    ///
    /// The caller has already been checked against the channel: this only
    /// stores the choice, under the channel's own organization.
    #[transactional(notification_preference, channel)]
    pub async fn set_notification_level(
        &self,
        user_id: UserId,
        channel_id: ChannelId,
        level: NotificationLevel,
    ) -> Result<(), CoreError> {
        let mut channel_repository = channel_repository;
        let channel = channel_repository
            .find_by_id(channel_id)
            .await?
            .ok_or(CoreError::NotFound)?;
        let mut notification_preference_repository = notification_preference_repository;
        notification_preference_repository
            .upsert(channel.organization_id, user_id, channel_id, level)
            .await
    }

    /// The caller's explicit choices; a channel not listed has its default.
    #[transactional(notification_preference)]
    pub async fn list_notification_levels(
        &self,
        user_id: UserId,
        organization_id: OrganizationId,
    ) -> Result<HashMap<ChannelId, NotificationLevel>, CoreError> {
        let mut notification_preference_repository = notification_preference_repository;
        notification_preference_repository
            .list_for_user(organization_id, user_id)
            .await
    }

    /// What the caller has to read: mentions, and the unread messages of the
    /// channels they follow, newest first.
    ///
    /// `visible` are the channels the caller may see; direct messages are
    /// added here from the caller's own pairs, so nobody's conversation can
    /// show up in somebody else's activity.
    #[transactional(
        notification,
        notification_preference,
        read_state,
        message,
        direct_channel
    )]
    pub async fn list_activity(
        &self,
        user_id: UserId,
        organization_id: OrganizationId,
        visible: Vec<Channel>,
        limit: i64,
    ) -> Result<Vec<ActivityEntry>, CoreError> {
        let mut direct_channel_repository = direct_channel_repository;
        let mut notification_preference_repository = notification_preference_repository;
        let mut message_repository = message_repository;

        let mut types: HashMap<ChannelId, ChannelType> = visible
            .iter()
            .map(|channel| (channel.id, channel.channel_type))
            .collect();
        for direct in direct_channel_repository
            .list_for_user(organization_id, user_id)
            .await?
        {
            types.insert(direct.channel.id, ChannelType::Direct);
        }
        let chosen = notification_preference_repository
            .list_for_user(organization_id, user_id)
            .await?;
        let level_of = |channel_id: ChannelId| {
            types
                .get(&channel_id)
                .map(|channel_type| effective_level(&chosen, channel_id, *channel_type))
        };

        let mut entries = Vec::new();
        let mut mentioned: HashSet<MessageId> = HashSet::new();

        for notification in notification_repository
            .list(user_id, organization_id, false, None, limit)
            .await?
        {
            let Some(level) = level_of(notification.channel_id) else {
                continue;
            };
            if level == NotificationLevel::Nothing {
                continue;
            }
            let Some(message) = message_repository
                .find_by_id(notification.message_id)
                .await?
            else {
                continue;
            };
            mentioned.insert(message.id);
            entries.push(ActivityEntry {
                kind: match notification.kind {
                    NotificationKind::Mention => ActivityKind::Mention,
                    NotificationKind::Reply => ActivityKind::Reply,
                },
                channel_id: notification.channel_id,
                created_at: message.created_at,
                message,
                notification_id: Some(notification.id),
                unread_count: None,
                read: notification.read_at.is_some(),
            });
        }

        for unread in read_state_repository
            .unread_summaries(user_id, organization_id)
            .await?
        {
            if level_of(unread.channel_id) != Some(NotificationLevel::All) {
                continue;
            }
            let Some(message) = message_repository
                .find_by_id(unread.first_unread_message_id)
                .await?
            else {
                continue;
            };
            entries.push(ActivityEntry {
                kind: ActivityKind::Unread,
                channel_id: unread.channel_id,
                created_at: message.created_at,
                message,
                notification_id: None,
                unread_count: Some(unread.unread_count),
                read: false,
            });
        }

        entries.sort_by_key(|entry| std::cmp::Reverse(entry.created_at));
        entries.truncate(usize::try_from(limit.max(0)).unwrap_or(usize::MAX));
        Ok(entries)
    }
}
