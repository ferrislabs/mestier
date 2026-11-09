//! What the chat publishes to the automation backbone.
//!
//! Two events: [`ChatMessagePosted`] and [`ChatChannelCreated`]. The gateway's
//! own realtime events (`discord::DomainEvent`) are a different thing — they
//! feed open browser tabs, not workflows.
//!
//! Direct messages never reach here: they are private between two people, and
//! an automation is configured by an organization administrator. The message
//! payload also carries no `content`, so a workflow cannot read a private
//! channel through its trigger.

use discord::{Channel, Message};
use events::{DomainEvent, EventDescriptor, EventSubject};
use serde_json::{Value, json};

pub struct ChatMessagePosted {
    pub message: Message,
}

impl DomainEvent for ChatMessagePosted {
    fn name(&self) -> &'static str {
        "discussion.message.sended"
    }

    fn version(&self) -> u16 {
        1
    }

    fn subject(&self) -> EventSubject {
        EventSubject::new("chat_message", self.message.id.0)
    }

    fn payload(&self) -> Value {
        let message = &self.message;
        json!({
            "message_id": message.id.0,
            "organization_id": message.organization_id.0,
            "channel_id": message.channel_id.0,
            "author_type": message.author_type.as_str(),
            "author_user_id": message.author_user_id.map(|id| id.0),
            "mentions_everyone": message.mention_everyone,
            "mentioned_user_ids": message.mention_user_ids.iter().map(|id| id.0).collect::<Vec<_>>(),
            "created_at": message.created_at,
        })
    }
}

pub struct ChatChannelCreated {
    pub channel: Channel,
}

impl DomainEvent for ChatChannelCreated {
    fn name(&self) -> &'static str {
        "discussion.channel.created"
    }

    fn version(&self) -> u16 {
        1
    }

    fn subject(&self) -> EventSubject {
        EventSubject::new("chat_channel", self.channel.id.0)
    }

    fn payload(&self) -> Value {
        let channel = &self.channel;
        json!({
            "channel_id": channel.id.0,
            "organization_id": channel.organization_id.0,
            "channel_type": channel.channel_type.as_str(),
            "name": channel.name,
            "category_id": channel.category_id.map(|id| id.0),
            "project_id": channel.project_id.map(|id| id.0),
            "created_at": channel.created_at,
        })
    }
}

/// Test-only, mirrors `quote::events::emitted_events`.
#[cfg(test)]
pub fn emitted_events() -> Vec<(&'static str, u16)> {
    vec![
        ("discussion.message.sended", 1),
        ("discussion.channel.created", 1),
    ]
}

pub fn descriptors() -> Vec<EventDescriptor> {
    vec![
        EventDescriptor {
            name: "discussion.message.sended",
            version: 1,
            label: "Message posté dans un canal",
            subject_kind: "chat_message",
            payload_example: json!({
                "message_id": "018f3b2a-0000-7000-8000-000000000010",
                "organization_id": "018f3b2a-0000-7000-8000-000000000002",
                "channel_id": "018f3b2a-0000-7000-8000-000000000011",
                "author_type": "USER",
                "author_user_id": "018f3b2a-0000-7000-8000-000000000005",
                "mentions_everyone": false,
                "mentioned_user_ids": [],
                "created_at": "2026-11-08T09:00:00Z",
            }),
        },
        EventDescriptor {
            name: "discussion.channel.created",
            version: 1,
            label: "Canal créé",
            subject_kind: "chat_channel",
            payload_example: json!({
                "channel_id": "018f3b2a-0000-7000-8000-000000000011",
                "organization_id": "018f3b2a-0000-7000-8000-000000000002",
                "channel_type": "TEXT",
                "name": "chantier-dupont",
                "category_id": null,
                "project_id": null,
                "created_at": "2026-11-08T09:00:00Z",
            }),
        },
    ]
}
