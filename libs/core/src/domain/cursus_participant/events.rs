use chrono::{DateTime, Utc};
use events::{DomainEvent, EventDescriptor, EventSubject};
use serde_json::{Value, json};

use crate::{Participant, UserId};

pub struct ParticipantInvited {
    pub participant: Participant,
    pub expires_at: DateTime<Utc>,
}

impl DomainEvent for ParticipantInvited {
    fn name(&self) -> &'static str {
        "cursus.participant.invited"
    }

    fn version(&self) -> u16 {
        1
    }

    fn subject(&self) -> EventSubject {
        EventSubject::new("participant", self.participant.id.0)
    }

    fn payload(&self) -> Value {
        json!({
            "participant_id": self.participant.id.0,
            "organization_id": self.participant.organization_id.0,
            "email": self.participant.email,
            "expires_at": self.expires_at,
        })
    }
}

pub struct ParticipantActivated {
    pub participant: Participant,
    pub user_id: UserId,
    pub activated_at: DateTime<Utc>,
}

impl DomainEvent for ParticipantActivated {
    fn name(&self) -> &'static str {
        "cursus.participant.activated"
    }

    fn version(&self) -> u16 {
        1
    }

    fn subject(&self) -> EventSubject {
        EventSubject::new("participant", self.participant.id.0)
    }

    fn payload(&self) -> Value {
        json!({
            "participant_id": self.participant.id.0,
            "organization_id": self.participant.organization_id.0,
            "user_id": self.user_id.0,
            "activated_at": self.activated_at,
        })
    }
}

#[cfg(test)]
pub fn emitted_events() -> Vec<(&'static str, u16)> {
    vec![
        ("cursus.participant.invited", 1),
        ("cursus.participant.activated", 1),
    ]
}

pub fn descriptors() -> Vec<EventDescriptor> {
    vec![
        EventDescriptor {
            name: "cursus.participant.invited",
            version: 1,
            label: "Participant invité",
            subject_kind: "participant",
            payload_example: json!({
                "participant_id": "018f3b2a-0000-7000-8000-000000000001",
                "organization_id": "018f3b2a-0000-7000-8000-000000000002",
                "email": "apprenti@example.com",
                "expires_at": "2026-11-23T09:00:00Z",
            }),
        },
        EventDescriptor {
            name: "cursus.participant.activated",
            version: 1,
            label: "Participant activé",
            subject_kind: "participant",
            payload_example: json!({
                "participant_id": "018f3b2a-0000-7000-8000-000000000001",
                "organization_id": "018f3b2a-0000-7000-8000-000000000002",
                "user_id": "018f3b2a-0000-7000-8000-000000000003",
                "activated_at": "2026-11-09T09:00:00Z",
            }),
        },
    ]
}
