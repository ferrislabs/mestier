use serde::{Deserialize, Serialize};
use std::{fmt::Display, str::FromStr};
use utoipa::ToSchema;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ChannelType {
    Text,
    Thread,
}

impl ChannelType {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Text => "TEXT",
            Self::Thread => "THREAD",
        }
    }
}

impl Display for ChannelType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl FromStr for ChannelType {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "TEXT" => Ok(Self::Text),
            "THREAD" => Ok(Self::Thread),
            other => Err(format!("invalid channel type `{other}`")),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AuthorType {
    User,
    Webhook,
    System,
}

impl AuthorType {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::User => "USER",
            Self::Webhook => "WEBHOOK",
            Self::System => "SYSTEM",
        }
    }
}

impl Display for AuthorType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl FromStr for AuthorType {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "USER" => Ok(Self::User),
            "WEBHOOK" => Ok(Self::Webhook),
            "SYSTEM" => Ok(Self::System),
            other => Err(format!("invalid author type `{other}`")),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PresenceStatus {
    Online,
    Offline,
    Dnd,
}

impl PresenceStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Online => "ONLINE",
            Self::Offline => "OFFLINE",
            Self::Dnd => "DND",
        }
    }
}

impl Display for PresenceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl FromStr for PresenceStatus {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "ONLINE" => Ok(Self::Online),
            "OFFLINE" => Ok(Self::Offline),
            "DND" => Ok(Self::Dnd),
            other => Err(format!("invalid presence status `{other}`")),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum NotificationKind {
    Mention,
    Reply,
}

impl NotificationKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Mention => "MENTION",
            Self::Reply => "REPLY",
        }
    }
}

impl Display for NotificationKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl FromStr for NotificationKind {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "MENTION" => Ok(Self::Mention),
            "REPLY" => Ok(Self::Reply),
            other => Err(format!("invalid notification kind `{other}`")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn channel_type_parses_known_values() {
        assert_eq!("TEXT".parse::<ChannelType>().unwrap(), ChannelType::Text);
        assert_eq!(
            "THREAD".parse::<ChannelType>().unwrap(),
            ChannelType::Thread
        );
    }

    #[test]
    fn channel_type_rejects_unknown() {
        assert!("VOICE".parse::<ChannelType>().is_err());
    }

    #[test]
    fn channel_type_display_matches_as_str() {
        assert_eq!(ChannelType::Text.to_string(), "TEXT");
        assert_eq!(ChannelType::Thread.to_string(), "THREAD");
    }

    #[test]
    fn author_type_parses_known_values() {
        assert_eq!("USER".parse::<AuthorType>().unwrap(), AuthorType::User);
        assert_eq!(
            "WEBHOOK".parse::<AuthorType>().unwrap(),
            AuthorType::Webhook
        );
        assert_eq!("SYSTEM".parse::<AuthorType>().unwrap(), AuthorType::System);
    }

    #[test]
    fn author_type_rejects_unknown() {
        assert!("BOT".parse::<AuthorType>().is_err());
    }

    #[test]
    fn presence_status_parses_known_values() {
        assert_eq!(
            "ONLINE".parse::<PresenceStatus>().unwrap(),
            PresenceStatus::Online
        );
        assert_eq!(
            "OFFLINE".parse::<PresenceStatus>().unwrap(),
            PresenceStatus::Offline
        );
        assert_eq!(
            "DND".parse::<PresenceStatus>().unwrap(),
            PresenceStatus::Dnd
        );
    }

    #[test]
    fn presence_status_rejects_unknown() {
        assert!("AWAY".parse::<PresenceStatus>().is_err());
    }

    #[test]
    fn presence_status_display_matches_as_str() {
        assert_eq!(PresenceStatus::Dnd.to_string(), "DND");
    }

    #[test]
    fn notification_kind_parses_known_values() {
        assert_eq!(
            "MENTION".parse::<NotificationKind>().unwrap(),
            NotificationKind::Mention
        );
        assert_eq!(
            "REPLY".parse::<NotificationKind>().unwrap(),
            NotificationKind::Reply
        );
    }

    #[test]
    fn notification_kind_rejects_unknown() {
        assert!("mention".parse::<NotificationKind>().is_err());
        assert!("DM".parse::<NotificationKind>().is_err());
    }

    #[test]
    fn notification_kind_display_matches_as_str() {
        assert_eq!(NotificationKind::Mention.to_string(), "MENTION");
        assert_eq!(NotificationKind::Reply.to_string(), "REPLY");
    }
}
