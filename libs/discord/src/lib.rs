// Discord domain crate — pure domain, no infra, no core dependency.
// Depends only on `common`.

pub mod enums;
pub mod ids;

pub use enums::{AuthorType, ChannelType, PresenceStatus};
pub use ids::{AttachmentId, CategoryId, ChannelId, MessageId, ReactionId, WebhookId};

pub mod components;

pub mod domain;

pub use domain::{
    Attachment, Category, Channel, Message, Presence, Reaction, ReactionCount, Webhook,
};

pub mod mentions;

pub use mentions::{ParsedMentions, parse_mentions};

pub mod events;
pub use events::{DomainEvent, EventPublisher};

pub use common::{OrganizationId, RoleId, UserId};

// Flat re-exports of each aggregate's port, service, and commands so consumers
// (Plan 2 infra/application) can import them directly from the crate root.
pub use domain::category::{
    CategoryRepository, CategoryService, CreateCategoryCommand, UpdateCategoryCommand,
};
pub use domain::channel::{
    ChannelRepository, ChannelService, CreateChannelCommand, CreateThreadCommand,
    UpdateChannelCommand, UpdateThreadCommand,
};
pub use domain::message::{
    AddReactionCommand, AttachmentInput, CreateMessageCommand, MessageAuthor, MessageRepository,
    MessageService, RemoveReactionCommand, UpdateMessageCommand,
};
pub use domain::presence::{
    PresenceRepository, PresenceService, SetPresenceCommand, StartTypingCommand,
};
pub use domain::webhook::{
    CreateWebhookCommand, ExecuteWebhookCommand, UpdateWebhookCommand, WebhookRepository,
    WebhookService,
};
