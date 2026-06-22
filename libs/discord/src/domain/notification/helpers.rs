use std::collections::HashSet;

use common::UserId;

use crate::{PresenceStatus, domain::Message};

pub fn notification_should_deliver(status: Option<PresenceStatus>) -> bool {
    !matches!(status, Some(PresenceStatus::Dnd))
}

pub fn mention_notification_recipients(message: &Message) -> Vec<UserId> {
    let mut seen: HashSet<UserId> = HashSet::new();
    let mut recipients: Vec<UserId> = Vec::new();
    for &user_id in &message.mention_user_ids {
        if message.author_user_id == Some(user_id) {
            continue;
        }
        if seen.insert(user_id) {
            recipients.push(user_id);
        }
    }
    recipients
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ChannelId, MessageId, domain::Message, enums::AuthorType};
    use chrono::Utc;
    use common::OrganizationId;
    use uuid::Uuid;

    fn make_message(author_user_id: Option<UserId>, mention_user_ids: Vec<UserId>) -> Message {
        Message {
            id: MessageId(Uuid::new_v4()),
            organization_id: OrganizationId(Uuid::new_v4()),
            channel_id: ChannelId(Uuid::new_v4()),
            author_type: if author_user_id.is_some() {
                AuthorType::User
            } else {
                AuthorType::Webhook
            },
            author_user_id,
            author_webhook_id: None,
            content: String::new(),
            components: None,
            mention_user_ids,
            mention_role_ids: vec![],
            mention_channel_ids: vec![],
            mention_everyone: false,
            reactions: vec![],
            attachments: vec![],
            edited_at: None,
            created_at: Utc::now(),
        }
    }

    // --- notification_should_deliver ---

    #[test]
    fn should_deliver_when_status_is_online() {
        assert!(notification_should_deliver(Some(PresenceStatus::Online)));
    }

    #[test]
    fn should_deliver_when_status_is_offline() {
        assert!(notification_should_deliver(Some(PresenceStatus::Offline)));
    }

    #[test]
    fn should_not_deliver_when_status_is_dnd() {
        assert!(!notification_should_deliver(Some(PresenceStatus::Dnd)));
    }

    #[test]
    fn should_deliver_when_no_presence_row() {
        assert!(notification_should_deliver(None));
    }

    // --- mention_notification_recipients ---

    #[test]
    fn recipients_deduplicates_mention_user_ids() {
        let user_a = UserId(Uuid::new_v4());
        let user_b = UserId(Uuid::new_v4());
        let message = make_message(None, vec![user_a, user_b, user_a]);
        let recipients = mention_notification_recipients(&message);
        assert_eq!(recipients, vec![user_a, user_b]);
    }

    #[test]
    fn recipients_excludes_author_when_mentioned() {
        let author = UserId(Uuid::new_v4());
        let other = UserId(Uuid::new_v4());
        let message = make_message(Some(author), vec![author, other]);
        let recipients = mention_notification_recipients(&message);
        assert_eq!(recipients, vec![other]);
    }

    #[test]
    fn recipients_keeps_all_when_author_is_none_webhook() {
        let user_a = UserId(Uuid::new_v4());
        let user_b = UserId(Uuid::new_v4());
        // author_user_id is None (webhook author)
        let message = make_message(None, vec![user_a, user_b]);
        let recipients = mention_notification_recipients(&message);
        assert_eq!(recipients, vec![user_a, user_b]);
    }

    #[test]
    fn recipients_preserves_order_after_dedup() {
        let user_a = UserId(Uuid::new_v4());
        let user_b = UserId(Uuid::new_v4());
        let user_c = UserId(Uuid::new_v4());
        let message = make_message(None, vec![user_b, user_a, user_c, user_a, user_b]);
        let recipients = mention_notification_recipients(&message);
        assert_eq!(recipients, vec![user_b, user_a, user_c]);
    }

    #[test]
    fn recipients_returns_empty_when_no_mentions() {
        let message = make_message(Some(UserId(Uuid::new_v4())), vec![]);
        let recipients = mention_notification_recipients(&message);
        assert!(recipients.is_empty());
    }
}
