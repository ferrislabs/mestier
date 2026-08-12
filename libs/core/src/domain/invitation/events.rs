//! What the invitation flow publishes.
//!
//! Two events, one per side of the handshake: [`MemberInvited`] when a link
//! is issued, [`MemberJoined`] when it is redeemed. Neither payload carries
//! the token or its hash — a subscriber has no legitimate use for either,
//! and the whole point of hashing it at rest is that it never appears
//! anywhere else, outbound webhooks included.

use events::{DomainEvent, EventDescriptor, EventSubject};
use serde_json::{Value, json};

use crate::{Invitation, Member, domain::invitation::InvitationId};

pub struct MemberInvited {
    pub invitation: Invitation,
}

impl DomainEvent for MemberInvited {
    fn name(&self) -> &'static str {
        "member.invited"
    }

    fn version(&self) -> u16 {
        1
    }

    fn subject(&self) -> EventSubject {
        EventSubject::new("invitation", self.invitation.id.0)
    }

    fn payload(&self) -> Value {
        json!({
            "invitation_id": self.invitation.id.0,
            "organization_id": self.invitation.organization_id.0,
            "member_id": self.invitation.member_id.map(|id| id.0),
            "expires_at": self.invitation.expires_at,
            "created_by_user_id": self.invitation.created_by_user_id.0,
        })
    }
}

/// Emitted once, by `accept_invitation` — never by `add_member` or
/// organization bootstrap, which stay silent as today; extending emission
/// to those paths is a separate decision, not something this event's
/// existence implies.
pub struct MemberJoined {
    pub member: Member,
    pub invitation_id: InvitationId,
}

impl DomainEvent for MemberJoined {
    fn name(&self) -> &'static str {
        "member.joined"
    }

    fn version(&self) -> u16 {
        1
    }

    fn subject(&self) -> EventSubject {
        EventSubject::new("member", self.member.id.0)
    }

    fn payload(&self) -> Value {
        json!({
            "member_id": self.member.id.0,
            "organization_id": self.member.organization_id.0,
            "user_id": self.member.user_id.map(|id| id.0),
            "invitation_id": self.invitation_id.0,
            "joined_at": self.member.joined_at,
        })
    }
}

/// Test-only, mirrors `quote::events::emitted_events`: the drift check in
/// `domain::automation::catalogue` compares this against `descriptors()`.
#[cfg(test)]
pub fn emitted_events() -> Vec<(&'static str, u16)> {
    vec![("member.invited", 1), ("member.joined", 1)]
}

pub fn descriptors() -> Vec<EventDescriptor> {
    vec![
        EventDescriptor {
            name: "member.invited",
            version: 1,
            label: "Invitation envoyée",
            subject_kind: "invitation",
            payload_example: json!({
                "invitation_id": "018f3b2a-0000-7000-8000-000000000001",
                "organization_id": "018f3b2a-0000-7000-8000-000000000002",
                "member_id": "018f3b2a-0000-7000-8000-000000000003",
                "expires_at": "2026-08-19T09:00:00Z",
                "created_by_user_id": "018f3b2a-0000-7000-8000-000000000004",
            }),
        },
        EventDescriptor {
            name: "member.joined",
            version: 1,
            label: "Membre arrivé",
            subject_kind: "member",
            payload_example: json!({
                "member_id": "018f3b2a-0000-7000-8000-000000000003",
                "organization_id": "018f3b2a-0000-7000-8000-000000000002",
                "user_id": "018f3b2a-0000-7000-8000-000000000005",
                "invitation_id": "018f3b2a-0000-7000-8000-000000000001",
                "joined_at": "2026-08-12T09:00:00Z",
            }),
        },
    ]
}
