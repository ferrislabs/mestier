//! What the member aggregate publishes.
//!
//! One event today: [`MemberRemoved`], emitted when a seat leaves the
//! organization (`MestierUseCase::remove_member`). `add_member` and
//! organization bootstrap stay silent — see `crate::domain::invitation::events`
//! for why `member.joined` is not their concern either.

use events::{DomainEvent, EventDescriptor, EventSubject};
use serde_json::{Value, json};

use crate::Member;

pub struct MemberRemoved {
    pub member: Member,
}

impl DomainEvent for MemberRemoved {
    fn name(&self) -> &'static str {
        "member.removed"
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
            "display_name": self.member.display_name(),
        })
    }
}

/// Test-only, mirrors `quote::events::emitted_events`.
#[cfg(test)]
pub fn emitted_events() -> Vec<(&'static str, u16)> {
    vec![("member.removed", 1)]
}

pub fn descriptors() -> Vec<EventDescriptor> {
    vec![EventDescriptor {
        name: "member.removed",
        version: 1,
        label: "Membre parti",
        subject_kind: "member",
        payload_example: json!({
            "member_id": "018f3b2a-0000-7000-8000-000000000003",
            "organization_id": "018f3b2a-0000-7000-8000-000000000002",
            "user_id": "018f3b2a-0000-7000-8000-000000000005",
            "display_name": "Parmantier Baptiste",
        }),
    }]
}
