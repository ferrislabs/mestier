use discord::{
    Category, CategoryId, Channel, ChannelId, DomainEvent, Message, MessageId, OrganizationId,
    Presence, UserId,
};
use serde::Serialize;

/// Wire-format gateway event sent to WebSocket clients.
/// JSON envelope: `{ "type": "MESSAGE_CREATE", "data": { ... } }`
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", content = "data", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum GatewayEvent {
    MessageCreate(Message),
    MessageUpdate(Message),
    MessageDelete {
        organization_id: OrganizationId,
        channel_id: ChannelId,
        message_id: MessageId,
    },
    ReactionAdd {
        organization_id: OrganizationId,
        message_id: MessageId,
        emoji: String,
        user_id: UserId,
    },
    ReactionRemove {
        organization_id: OrganizationId,
        message_id: MessageId,
        emoji: String,
        user_id: UserId,
    },
    CategoryCreate(Category),
    CategoryUpdate(Category),
    CategoryDelete {
        organization_id: OrganizationId,
        category_id: CategoryId,
    },
    ChannelCreate(Channel),
    ChannelUpdate(Channel),
    ChannelDelete {
        organization_id: OrganizationId,
        channel_id: ChannelId,
    },
    ThreadCreate(Channel),
    ThreadUpdate(Channel),
    ThreadDelete {
        organization_id: OrganizationId,
        channel_id: ChannelId,
    },
    PresenceUpdate(Presence),
    TypingStart {
        organization_id: OrganizationId,
        channel_id: ChannelId,
        user_id: UserId,
        ttl_ms: u64,
    },
}

/// How long a typing indicator stays active on the client side (milliseconds).
const TYPING_TTL_MS: u64 = 10_000;

/// Converts a `DomainEvent` into the wire representation.
/// `org_id` must be provided by the caller for events that do not carry an
/// `organization_id` inline (message deletes, reactions).
pub fn from_domain(event: DomainEvent, org_id: OrganizationId) -> GatewayEvent {
    match event {
        DomainEvent::MessageCreated(m) => GatewayEvent::MessageCreate(m),
        DomainEvent::MessageUpdated(m) => GatewayEvent::MessageUpdate(m),
        DomainEvent::MessageDeleted {
            channel_id,
            message_id,
        } => GatewayEvent::MessageDelete {
            organization_id: org_id,
            channel_id,
            message_id,
        },
        DomainEvent::ReactionAdded {
            message_id,
            emoji,
            user_id,
        } => GatewayEvent::ReactionAdd {
            organization_id: org_id,
            message_id,
            emoji,
            user_id,
        },
        DomainEvent::ReactionRemoved {
            message_id,
            emoji,
            user_id,
        } => GatewayEvent::ReactionRemove {
            organization_id: org_id,
            message_id,
            emoji,
            user_id,
        },
        DomainEvent::CategoryCreated(c) => GatewayEvent::CategoryCreate(c),
        DomainEvent::CategoryUpdated(c) => GatewayEvent::CategoryUpdate(c),
        DomainEvent::CategoryDeleted {
            organization_id,
            category_id,
        } => GatewayEvent::CategoryDelete {
            organization_id,
            category_id,
        },
        DomainEvent::ChannelCreated(c) => GatewayEvent::ChannelCreate(c),
        DomainEvent::ChannelUpdated(c) => GatewayEvent::ChannelUpdate(c),
        DomainEvent::ChannelDeleted {
            organization_id,
            channel_id,
        } => GatewayEvent::ChannelDelete {
            organization_id,
            channel_id,
        },
        DomainEvent::ThreadCreated(c) => GatewayEvent::ThreadCreate(c),
        DomainEvent::ThreadUpdated(c) => GatewayEvent::ThreadUpdate(c),
        DomainEvent::ThreadDeleted {
            organization_id,
            channel_id,
        } => GatewayEvent::ThreadDelete {
            organization_id,
            channel_id,
        },
        DomainEvent::PresenceUpdated {
            organization_id,
            user_id,
            status,
        } => GatewayEvent::PresenceUpdate(Presence {
            organization_id,
            user_id,
            status,
            updated_at: chrono::Utc::now(),
        }),
        DomainEvent::TypingStarted {
            organization_id,
            channel_id,
            user_id,
        } => GatewayEvent::TypingStart {
            organization_id,
            channel_id,
            user_id,
            ttl_ms: TYPING_TTL_MS,
        },
    }
}
