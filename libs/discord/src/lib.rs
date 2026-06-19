// Discord domain crate — pure domain, no infra, no core dependency.
// Depends only on `common`.

pub mod enums;
pub mod ids;

pub use enums::{AuthorType, ChannelType, PresenceStatus};
pub use ids::{CategoryId, ChannelId, MessageId, ReactionId, WebhookId};

pub mod components;
