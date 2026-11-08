//! What the absence aggregate publishes.
//!
//! [`AbsenceRequested`] on every creation, whatever the status it lands in,
//! then one named event per decision. An absence created already approved
//! announces both, in that order.

use events::{DomainEvent, EventDescriptor, EventSubject};
use serde_json::{Value, json};

use crate::{Absence, AbsenceStatus};

pub struct AbsenceRequested {
    pub absence: Absence,
}

impl DomainEvent for AbsenceRequested {
    fn name(&self) -> &'static str {
        "absence.requested"
    }

    fn version(&self) -> u16 {
        1
    }

    fn subject(&self) -> EventSubject {
        EventSubject::new("absence", self.absence.id.0)
    }

    fn payload(&self) -> Value {
        json!({ "absence": absence_payload(&self.absence) })
    }
}

/// One decision on an absence, named after where it lands.
pub struct AbsenceDecided {
    pub absence: Absence,
    pub from: AbsenceStatus,
}

impl AbsenceDecided {
    pub fn event_name(to: AbsenceStatus) -> Option<&'static str> {
        match to {
            AbsenceStatus::Approved => Some("absence.approved"),
            AbsenceStatus::Rejected => Some("absence.rejected"),
            AbsenceStatus::Cancelled => Some("absence.cancelled"),
            AbsenceStatus::Pending => None,
        }
    }
}

impl DomainEvent for AbsenceDecided {
    fn name(&self) -> &'static str {
        Self::event_name(self.absence.status).expect("a nameless decision is never emitted")
    }

    fn version(&self) -> u16 {
        1
    }

    fn subject(&self) -> EventSubject {
        EventSubject::new("absence", self.absence.id.0)
    }

    fn payload(&self) -> Value {
        json!({
            "absence": absence_payload(&self.absence),
            "from": self.from.as_str(),
            "to": self.absence.status.as_str(),
        })
    }
}

/// The serialized domain model, never the database row: renaming a column must
/// not reach a subscriber's automation.
fn absence_payload(absence: &Absence) -> Value {
    json!({
        "id": absence.id.0,
        "organization_id": absence.organization_id.0,
        "member_id": absence.member_id.0,
        "kind": absence.kind.as_str(),
        "type_id": absence.type_id.0,
        "starts_at": absence.starts_at,
        "ends_at": absence.ends_at,
        "all_day": absence.all_day,
        "start_date": absence.start_date,
        "start_part": absence.start_part.as_str(),
        "end_date": absence.end_date,
        "end_part": absence.end_part.as_str(),
        "status": absence.status.as_str(),
        "note": absence.note,
        "proof_file_key": absence.proof_file_key,
        "created_by_member_id": absence.created_by_member_id.map(|id| id.0),
        "decided_by_member_id": absence.decided_by_member_id.map(|id| id.0),
        "decided_at": absence.decided_at,
        "decision_comment": absence.decision_comment,
        "created_at": absence.created_at,
        "updated_at": absence.updated_at,
    })
}

/// Test-only, mirrors `quote::events::emitted_events`.
#[cfg(test)]
pub fn emitted_events() -> Vec<(&'static str, u16)> {
    vec![
        ("absence.requested", 1),
        ("absence.approved", 1),
        ("absence.rejected", 1),
        ("absence.cancelled", 1),
    ]
}

pub fn descriptors() -> Vec<EventDescriptor> {
    let absence_example = json!({
        "id": "018f3b2a-0000-7000-8000-000000000010",
        "organization_id": "018f3b2a-0000-7000-8000-000000000002",
        "member_id": "018f3b2a-0000-7000-8000-000000000004",
        "kind": "LEAVE",
        "type_id": "018f3b2a-0000-7000-8000-000000000011",
        "starts_at": "2026-12-21T07:00:00Z",
        "ends_at": "2026-12-23T16:00:00Z",
        "all_day": true,
        "start_date": "2026-12-22",
        "start_part": "MORNING",
        "end_date": "2026-12-23",
        "end_part": "AFTERNOON",
        "status": "PENDING",
        "note": "Congés de fin d'année",
        "proof_file_key": null,
        "created_by_member_id": "018f3b2a-0000-7000-8000-000000000004",
        "decided_by_member_id": null,
        "decided_at": null,
        "decision_comment": null,
        "created_at": "2026-11-08T09:00:00Z",
        "updated_at": "2026-11-08T09:00:00Z",
    });
    let decided = |status: &str, comment: Value| {
        let mut absence = absence_example.clone();
        absence["status"] = json!(status);
        absence["decided_by_member_id"] = json!("018f3b2a-0000-7000-8000-000000000005");
        absence["decided_at"] = json!("2026-11-09T09:00:00Z");
        absence["decision_comment"] = comment;
        absence
    };

    vec![
        EventDescriptor {
            name: "absence.requested",
            version: 1,
            label: "Absence demandée",
            subject_kind: "absence",
            payload_example: json!({ "absence": absence_example }),
        },
        EventDescriptor {
            name: "absence.approved",
            version: 1,
            label: "Absence approuvée",
            subject_kind: "absence",
            payload_example: json!({
                "absence": decided("APPROVED", Value::Null),
                "from": "PENDING",
                "to": "APPROVED",
            }),
        },
        EventDescriptor {
            name: "absence.rejected",
            version: 1,
            label: "Absence refusée",
            subject_kind: "absence",
            payload_example: json!({
                "absence": decided("REJECTED", json!("Période de forte activité")),
                "from": "PENDING",
                "to": "REJECTED",
            }),
        },
        EventDescriptor {
            name: "absence.cancelled",
            version: 1,
            label: "Absence annulée",
            subject_kind: "absence",
            payload_example: json!({
                "absence": decided("CANCELLED", Value::Null),
                "from": "PENDING",
                "to": "CANCELLED",
            }),
        },
    ]
}
