// Discord domain crate — pure domain, no infra, no core dependency.
// Depends only on `common`.

pub mod enums;
pub mod ids;

pub use enums::{AuthorType, ChannelType, PresenceStatus};
pub use ids::{CategoryId, ChannelId, MessageId, ReactionId, WebhookId};

pub mod components;

pub mod domain;

pub use domain::{Category, Channel, Message, Presence, Reaction, ReactionCount, Webhook};

pub mod mentions;

pub use mentions::{ParsedMentions, parse_mentions};
