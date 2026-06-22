use common::UserId;

use crate::{PresenceStatus, domain::Message};

pub fn notification_should_deliver(status: Option<PresenceStatus>) -> bool {
    !matches!(status, Some(PresenceStatus::Dnd))
}

pub fn mention_notification_recipients(message: &Message) -> Vec<UserId> {
    let mut seen = std::collections::HashSet::new();
    let mut recipients = Vec::new();
    for user_id in &message.mention_user_ids {
        if Some(user_id) == message.author_user_id.as_ref() {
            continue;
        }
        if seen.insert(*user_id) {
            recipients.push(*user_id);
        }
    }
    recipients
}
