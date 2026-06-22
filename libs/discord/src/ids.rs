use serde::{Deserialize, Serialize};
use std::{fmt::Display, str::FromStr};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema)]
pub struct CategoryId(pub Uuid);

impl FromStr for CategoryId {
    type Err = uuid::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Uuid::from_str(s).map(CategoryId)
    }
}

impl Display for CategoryId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema)]
pub struct ChannelId(pub Uuid);

impl FromStr for ChannelId {
    type Err = uuid::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Uuid::from_str(s).map(ChannelId)
    }
}

impl Display for ChannelId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema)]
pub struct MessageId(pub Uuid);

impl FromStr for MessageId {
    type Err = uuid::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Uuid::from_str(s).map(MessageId)
    }
}

impl Display for MessageId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema)]
pub struct ReactionId(pub Uuid);

impl FromStr for ReactionId {
    type Err = uuid::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Uuid::from_str(s).map(ReactionId)
    }
}

impl Display for ReactionId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema)]
pub struct WebhookId(pub Uuid);

impl FromStr for WebhookId {
    type Err = uuid::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Uuid::from_str(s).map(WebhookId)
    }
}

impl Display for WebhookId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema)]
pub struct AttachmentId(pub Uuid);

impl FromStr for AttachmentId {
    type Err = uuid::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Uuid::from_str(s).map(AttachmentId)
    }
}

impl Display for AttachmentId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;
    use uuid::Uuid;

    #[test]
    fn category_id_round_trips() {
        let uuid = Uuid::new_v4();
        let id = CategoryId::from_str(&uuid.to_string()).unwrap();
        assert_eq!(id.0, uuid);
        assert_eq!(id.to_string(), uuid.to_string());
    }

    #[test]
    fn channel_id_round_trips() {
        let uuid = Uuid::new_v4();
        let id = ChannelId::from_str(&uuid.to_string()).unwrap();
        assert_eq!(id.0, uuid);
    }

    #[test]
    fn message_id_round_trips() {
        let uuid = Uuid::new_v4();
        let id = MessageId::from_str(&uuid.to_string()).unwrap();
        assert_eq!(id.0, uuid);
    }

    #[test]
    fn reaction_id_round_trips() {
        let uuid = Uuid::new_v4();
        let id = ReactionId::from_str(&uuid.to_string()).unwrap();
        assert_eq!(id.0, uuid);
    }

    #[test]
    fn webhook_id_round_trips() {
        let uuid = Uuid::new_v4();
        let id = WebhookId::from_str(&uuid.to_string()).unwrap();
        assert_eq!(id.0, uuid);
    }

    #[test]
    fn all_ids_reject_invalid_uuid() {
        assert!(AttachmentId::from_str("not-a-uuid").is_err());
        assert!(CategoryId::from_str("not-a-uuid").is_err());
        assert!(ChannelId::from_str("not-a-uuid").is_err());
        assert!(MessageId::from_str("not-a-uuid").is_err());
        assert!(ReactionId::from_str("not-a-uuid").is_err());
        assert!(WebhookId::from_str("not-a-uuid").is_err());
    }

    #[test]
    fn attachment_id_round_trips() {
        let uuid = Uuid::new_v4();
        let id = AttachmentId::from_str(&uuid.to_string()).unwrap();
        assert_eq!(id.0, uuid);
        assert_eq!(id.to_string(), uuid.to_string());
    }

    #[test]
    fn attachment_id_rejects_invalid_uuid() {
        assert!(AttachmentId::from_str("not-a-uuid").is_err());
    }
}
