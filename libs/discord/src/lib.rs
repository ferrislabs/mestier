// Discord domain crate — pure domain, no infra, no core dependency.
// Depends only on `common`.

pub mod ids;

pub use ids::{CategoryId, ChannelId, MessageId, ReactionId, WebhookId};
