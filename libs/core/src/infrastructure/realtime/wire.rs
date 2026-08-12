use discord::{
    Category, CategoryId, Channel, ChannelId, DomainEvent, Message, MessageId, Notification,
    OrganizationId, Presence, UserId,
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
        channel_id: ChannelId,
        message_id: MessageId,
        emoji: String,
        user_id: UserId,
    },
    ReactionRemove {
        organization_id: OrganizationId,
        channel_id: ChannelId,
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
    ChannelRead {
        organization_id: OrganizationId,
        channel_id: ChannelId,
        user_id: UserId,
        last_read_message_id: Option<MessageId>,
    },
    NotificationCreate(Notification),
}

/// How long a typing indicator stays active on the client side (milliseconds).
const TYPING_TTL_MS: u64 = 10_000;

/// Converts a `DomainEvent` into the wire representation.
///
/// Every `DomainEvent` variant carries its own `organization_id`, so the
/// conversion is total and needs no caller-supplied context. This is deliberate:
/// the org used to be a `flush(org_id)` argument re-stamped onto the three
/// variants that lacked one, which made a cross-organization delivery a
/// hand-written argument away rather than something the types ruled out.
pub fn from_domain(event: DomainEvent) -> GatewayEvent {
    match event {
        DomainEvent::MessageCreated(m) => GatewayEvent::MessageCreate(m),
        DomainEvent::MessageUpdated(m) => GatewayEvent::MessageUpdate(m),
        DomainEvent::MessageDeleted {
            organization_id,
            channel_id,
            message_id,
        } => GatewayEvent::MessageDelete {
            organization_id,
            channel_id,
            message_id,
        },
        DomainEvent::ReactionAdded {
            organization_id,
            channel_id,
            message_id,
            emoji,
            user_id,
        } => GatewayEvent::ReactionAdd {
            organization_id,
            channel_id,
            message_id,
            emoji,
            user_id,
        },
        DomainEvent::ReactionRemoved {
            organization_id,
            channel_id,
            message_id,
            emoji,
            user_id,
        } => GatewayEvent::ReactionRemove {
            organization_id,
            channel_id,
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
        DomainEvent::ChannelRead {
            organization_id,
            channel_id,
            user_id,
            last_read_message_id,
        } => GatewayEvent::ChannelRead {
            organization_id,
            channel_id,
            user_id,
            last_read_message_id,
        },
        DomainEvent::NotificationCreated(n) => GatewayEvent::NotificationCreate(n),
    }
}

impl GatewayEvent {
    /// The organization this event belongs to — the hub channel it is broadcast on.
    ///
    /// Every variant answers from data it already carries, so routing cannot
    /// disagree with the payload: an event delivered to an organization is by
    /// construction an event stamped with that organization.
    pub fn organization_id(&self) -> OrganizationId {
        match self {
            GatewayEvent::MessageCreate(m) | GatewayEvent::MessageUpdate(m) => m.organization_id,
            GatewayEvent::CategoryCreate(c) | GatewayEvent::CategoryUpdate(c) => c.organization_id,
            GatewayEvent::ChannelCreate(c)
            | GatewayEvent::ChannelUpdate(c)
            | GatewayEvent::ThreadCreate(c)
            | GatewayEvent::ThreadUpdate(c) => c.organization_id,
            GatewayEvent::PresenceUpdate(p) => p.organization_id,
            GatewayEvent::NotificationCreate(n) => n.organization_id,
            GatewayEvent::MessageDelete {
                organization_id, ..
            }
            | GatewayEvent::ReactionAdd {
                organization_id, ..
            }
            | GatewayEvent::ReactionRemove {
                organization_id, ..
            }
            | GatewayEvent::CategoryDelete {
                organization_id, ..
            }
            | GatewayEvent::ChannelDelete {
                organization_id, ..
            }
            | GatewayEvent::ThreadDelete {
                organization_id, ..
            }
            | GatewayEvent::TypingStart {
                organization_id, ..
            }
            | GatewayEvent::ChannelRead {
                organization_id, ..
            } => *organization_id,
        }
    }

    /// The channel this event concerns, when it concerns one.
    ///
    /// `None` means the event is organization-scoped (categories, presence) and
    /// carries nothing a channel-level permission could gate. The gateway uses
    /// this to decide whether an event must clear the member's visible-channel
    /// set before it reaches the socket — so a variant that returns `None`
    /// bypasses that check, and adding a channel-scoped variant that forgets to
    /// answer here would leak it. Thread events answer with the thread's own id,
    /// matching how `list_visible_channels` builds the set.
    pub fn channel_id(&self) -> Option<ChannelId> {
        match self {
            GatewayEvent::MessageCreate(m) | GatewayEvent::MessageUpdate(m) => Some(m.channel_id),
            GatewayEvent::ChannelCreate(c)
            | GatewayEvent::ChannelUpdate(c)
            | GatewayEvent::ThreadCreate(c)
            | GatewayEvent::ThreadUpdate(c) => Some(c.id),
            GatewayEvent::NotificationCreate(n) => Some(n.channel_id),
            GatewayEvent::MessageDelete { channel_id, .. }
            | GatewayEvent::ReactionAdd { channel_id, .. }
            | GatewayEvent::ReactionRemove { channel_id, .. }
            | GatewayEvent::ChannelDelete { channel_id, .. }
            | GatewayEvent::ThreadDelete { channel_id, .. }
            | GatewayEvent::TypingStart { channel_id, .. }
            | GatewayEvent::ChannelRead { channel_id, .. } => Some(*channel_id),
            GatewayEvent::CategoryCreate(_)
            | GatewayEvent::CategoryUpdate(_)
            | GatewayEvent::CategoryDelete { .. }
            | GatewayEvent::PresenceUpdate(_) => None,
        }
    }

    /// Returns `Some(user_id)` for user-private events (currently only `ChannelRead`).
    /// The gateway dispatch loop uses this to skip forwarding to other users' sockets.
    pub fn target_user(&self) -> Option<UserId> {
        match self {
            GatewayEvent::ChannelRead { user_id, .. } => Some(*user_id),
            GatewayEvent::NotificationCreate(n) => Some(n.user_id),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    fn channel_read_event() -> GatewayEvent {
        let org = OrganizationId(Uuid::from_u128(1));
        let ch = ChannelId(Uuid::from_u128(2));
        let u = UserId(Uuid::from_u128(3));
        GatewayEvent::ChannelRead {
            organization_id: org,
            channel_id: ch,
            user_id: u,
            last_read_message_id: None,
        }
    }

    #[test]
    fn target_user_returns_some_for_channel_read() {
        let ev = channel_read_event();
        let uid = UserId(Uuid::from_u128(3));
        assert_eq!(ev.target_user(), Some(uid));
    }

    #[test]
    fn target_user_returns_none_for_other_events() {
        let org = OrganizationId(Uuid::from_u128(1));
        let ch = ChannelId(Uuid::from_u128(2));
        let msg = MessageId(Uuid::from_u128(9));
        let ev = GatewayEvent::MessageDelete {
            organization_id: org,
            channel_id: ch,
            message_id: msg,
        };
        assert_eq!(ev.target_user(), None);
    }

    #[test]
    fn target_user_returns_some_for_notification_create() {
        use discord::{
            ChannelId, MessageId, Notification, NotificationId, NotificationKind, OrganizationId,
            UserId,
        };
        let notif = Notification {
            id: NotificationId(Uuid::from_u128(10)),
            organization_id: OrganizationId(Uuid::from_u128(1)),
            user_id: UserId(Uuid::from_u128(42)),
            channel_id: ChannelId(Uuid::from_u128(2)),
            message_id: MessageId(Uuid::from_u128(9)),
            kind: NotificationKind::Mention,
            read_at: None,
            created_at: chrono::Utc::now(),
        };
        let ev = GatewayEvent::NotificationCreate(notif);
        assert_eq!(ev.target_user(), Some(UserId(Uuid::from_u128(42))));
    }

    /// `channel_id()` gates the gateway's permission filter: a variant that
    /// answers `None` skips the visible-channel check entirely. So every
    /// channel-scoped variant must answer `Some`, and answer with the *right*
    /// channel — a wrong id would gate against somebody else's permissions.
    #[test]
    fn channel_id_is_some_for_every_channel_scoped_event() {
        let org = OrganizationId(Uuid::from_u128(1));
        let ch = ChannelId(Uuid::from_u128(2));
        let user = UserId(Uuid::from_u128(3));
        let msg = MessageId(Uuid::from_u128(9));

        let cases: Vec<(&str, GatewayEvent)> = vec![
            (
                "MessageDelete",
                GatewayEvent::MessageDelete {
                    organization_id: org,
                    channel_id: ch,
                    message_id: msg,
                },
            ),
            (
                "ReactionAdd",
                GatewayEvent::ReactionAdd {
                    organization_id: org,
                    channel_id: ch,
                    message_id: msg,
                    emoji: "👍".to_owned(),
                    user_id: user,
                },
            ),
            (
                "ReactionRemove",
                GatewayEvent::ReactionRemove {
                    organization_id: org,
                    channel_id: ch,
                    message_id: msg,
                    emoji: "👍".to_owned(),
                    user_id: user,
                },
            ),
            (
                "ChannelDelete",
                GatewayEvent::ChannelDelete {
                    organization_id: org,
                    channel_id: ch,
                },
            ),
            (
                "ThreadDelete",
                GatewayEvent::ThreadDelete {
                    organization_id: org,
                    channel_id: ch,
                },
            ),
            (
                "TypingStart",
                GatewayEvent::TypingStart {
                    organization_id: org,
                    channel_id: ch,
                    user_id: user,
                    ttl_ms: TYPING_TTL_MS,
                },
            ),
            (
                "ChannelRead",
                GatewayEvent::ChannelRead {
                    organization_id: org,
                    channel_id: ch,
                    user_id: user,
                    last_read_message_id: None,
                },
            ),
        ];

        for (name, ev) in cases {
            assert_eq!(
                ev.channel_id(),
                Some(ch),
                "{name} must be gated on its own channel"
            );
        }
    }

    /// The mirror of the above: these carry nothing a channel overwrite could
    /// gate, so they are organization-scoped by design rather than by oversight.
    #[test]
    fn channel_id_is_none_for_organization_scoped_events() {
        use discord::{CategoryId, Presence, PresenceStatus};
        let org = OrganizationId(Uuid::from_u128(1));

        let category_delete = GatewayEvent::CategoryDelete {
            organization_id: org,
            category_id: CategoryId(Uuid::from_u128(7)),
        };
        assert_eq!(category_delete.channel_id(), None);

        let presence = GatewayEvent::PresenceUpdate(Presence {
            organization_id: org,
            user_id: UserId(Uuid::from_u128(3)),
            status: PresenceStatus::Online,
            updated_at: chrono::Utc::now(),
        });
        assert_eq!(presence.channel_id(), None);
    }
}
