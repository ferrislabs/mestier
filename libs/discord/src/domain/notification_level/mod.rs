use std::collections::HashMap;
use std::fmt::{self, Display};
use std::future::Future;
use std::str::FromStr;

use common::{CoreError, OrganizationId, UserId};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::{ChannelId, ChannelType};

/// How much of a channel a person wants to be told about.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum NotificationLevel {
    /// Every message shows up in the activity.
    All,
    /// Only the messages that mention the person.
    Mentions,
    /// Nothing: no mention is recorded and the channel stays out of the activity.
    Nothing,
}

impl NotificationLevel {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::All => "ALL",
            Self::Mentions => "MENTIONS",
            Self::Nothing => "NOTHING",
        }
    }

    /// What applies when the person never chose: a direct message is meant
    /// for them, a channel is not.
    pub fn default_for(channel_type: ChannelType) -> Self {
        match channel_type {
            ChannelType::Direct => Self::All,
            _ => Self::Mentions,
        }
    }
}

impl Display for NotificationLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for NotificationLevel {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "ALL" => Ok(Self::All),
            "MENTIONS" => Ok(Self::Mentions),
            "NOTHING" => Ok(Self::Nothing),
            other => Err(format!("invalid notification level `{other}`")),
        }
    }
}

/// The level that applies to a channel, given the person's explicit choices.
pub fn effective_level(
    chosen: &HashMap<ChannelId, NotificationLevel>,
    channel_id: ChannelId,
    channel_type: ChannelType,
) -> NotificationLevel {
    chosen
        .get(&channel_id)
        .copied()
        .unwrap_or_else(|| NotificationLevel::default_for(channel_type))
}

#[cfg_attr(any(test, feature = "mock"), mockall::automock)]
pub trait NotificationPreferenceRepository: Send {
    fn upsert(
        &mut self,
        organization_id: OrganizationId,
        user_id: UserId,
        channel_id: ChannelId,
        level: NotificationLevel,
    ) -> impl Future<Output = Result<(), CoreError>> + Send;

    /// The explicit choices of one person in one organization.
    fn list_for_user(
        &mut self,
        organization_id: OrganizationId,
        user_id: UserId,
    ) -> impl Future<Output = Result<HashMap<ChannelId, NotificationLevel>, CoreError>> + Send;

    /// The explicit choices of several people on one channel.
    fn list_for_channel(
        &mut self,
        channel_id: ChannelId,
        user_ids: &[UserId],
    ) -> impl Future<Output = Result<HashMap<UserId, NotificationLevel>, CoreError>> + Send;
}

#[cfg(test)]
mod tests {
    use uuid::Uuid;

    use super::*;

    fn channel() -> ChannelId {
        ChannelId(Uuid::new_v4())
    }

    #[test]
    fn a_direct_message_defaults_to_everything_and_a_channel_to_mentions() {
        assert_eq!(
            NotificationLevel::default_for(ChannelType::Direct),
            NotificationLevel::All
        );
        assert_eq!(
            NotificationLevel::default_for(ChannelType::Text),
            NotificationLevel::Mentions
        );
        assert_eq!(
            NotificationLevel::default_for(ChannelType::Announcement),
            NotificationLevel::Mentions
        );
    }

    #[test]
    fn an_explicit_choice_beats_the_default() {
        let muted = channel();
        let chosen = HashMap::from([(muted, NotificationLevel::Nothing)]);

        assert_eq!(
            effective_level(&chosen, muted, ChannelType::Direct),
            NotificationLevel::Nothing
        );
        assert_eq!(
            effective_level(&chosen, channel(), ChannelType::Text),
            NotificationLevel::Mentions
        );
    }

    #[test]
    fn a_level_round_trips_through_its_text_form() {
        for level in [
            NotificationLevel::All,
            NotificationLevel::Mentions,
            NotificationLevel::Nothing,
        ] {
            assert_eq!(level.as_str().parse::<NotificationLevel>(), Ok(level));
        }
        assert!("LOUD".parse::<NotificationLevel>().is_err());
    }
}
