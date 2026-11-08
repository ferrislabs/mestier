//! What the extra-hours aggregate publishes.
//!
//! [`ExtraPeriodRequested`] on every declaration, whatever the status it lands
//! in, then one named event per decision. A period declared already approved
//! announces both, in that order.

use events::{DomainEvent, EventDescriptor, EventSubject};
use serde_json::{Value, json};

use crate::{ClockTime, ExtraPeriod, ExtraStatus};

pub struct ExtraPeriodRequested {
    pub period: ExtraPeriod,
}

impl DomainEvent for ExtraPeriodRequested {
    fn name(&self) -> &'static str {
        "extra_period.requested"
    }

    fn version(&self) -> u16 {
        1
    }

    fn subject(&self) -> EventSubject {
        EventSubject::new("extra_period", self.period.id.0)
    }

    fn payload(&self) -> Value {
        json!({ "extra_period": period_payload(&self.period) })
    }
}

/// One decision on an extra period, named after where it lands.
pub struct ExtraPeriodDecided {
    pub period: ExtraPeriod,
    pub from: ExtraStatus,
}

impl ExtraPeriodDecided {
    pub fn event_name(to: ExtraStatus) -> Option<&'static str> {
        match to {
            ExtraStatus::Approved => Some("extra_period.approved"),
            ExtraStatus::Rejected => Some("extra_period.rejected"),
            ExtraStatus::Cancelled => Some("extra_period.cancelled"),
            ExtraStatus::Pending => None,
        }
    }
}

impl DomainEvent for ExtraPeriodDecided {
    fn name(&self) -> &'static str {
        Self::event_name(self.period.status).expect("a nameless decision is never emitted")
    }

    fn version(&self) -> u16 {
        1
    }

    fn subject(&self) -> EventSubject {
        EventSubject::new("extra_period", self.period.id.0)
    }

    fn payload(&self) -> Value {
        json!({
            "extra_period": period_payload(&self.period),
            "from": self.from.as_str(),
            "to": self.period.status.as_str(),
        })
    }
}

fn clock(time: ClockTime) -> String {
    format!("{:02}:{:02}", time.minutes() / 60, time.minutes() % 60)
}

/// The serialized domain model, never the database row: renaming a column must
/// not reach a subscriber's automation.
fn period_payload(period: &ExtraPeriod) -> Value {
    json!({
        "id": period.id.0,
        "organization_id": period.organization_id.0,
        "member_id": period.member_id.0,
        "date": period.date,
        "start": clock(period.hours.start()),
        "end": clock(period.hours.end()),
        "lunch": period.hours.lunch().map(|lunch| json!({
            "start": clock(lunch.start()),
            "end": clock(lunch.end()),
        })),
        "worked_minutes": period.hours.worked_minutes(),
        "reason": period.reason.as_str(),
        "surcharge_id": period.surcharge_id.map(|id| id.0),
        "rate_bp": period.rate.get(),
        "status": period.status.as_str(),
        "created_by_member_id": period.created_by_member_id.map(|id| id.0),
        "decided_by_member_id": period.decided_by_member_id.map(|id| id.0),
        "decided_at": period.decided_at,
        "decision_comment": period.decision_comment,
        "created_at": period.created_at,
        "updated_at": period.updated_at,
    })
}

/// Test-only, mirrors `quote::events::emitted_events`.
#[cfg(test)]
pub fn emitted_events() -> Vec<(&'static str, u16)> {
    vec![
        ("extra_period.requested", 1),
        ("extra_period.approved", 1),
        ("extra_period.rejected", 1),
        ("extra_period.cancelled", 1),
    ]
}

pub fn descriptors() -> Vec<EventDescriptor> {
    let period_example = json!({
        "id": "018f3b2a-0000-7000-8000-000000000020",
        "organization_id": "018f3b2a-0000-7000-8000-000000000002",
        "member_id": "018f3b2a-0000-7000-8000-000000000004",
        "date": "2026-11-14",
        "start": "08:00",
        "end": "17:00",
        "lunch": { "start": "12:00", "end": "13:00" },
        "worked_minutes": 480,
        "reason": "Rattrapage du chantier",
        "surcharge_id": null,
        "rate_bp": 2500,
        "status": "PENDING",
        "created_by_member_id": "018f3b2a-0000-7000-8000-000000000004",
        "decided_by_member_id": null,
        "decided_at": null,
        "decision_comment": null,
        "created_at": "2026-11-08T09:00:00Z",
        "updated_at": "2026-11-08T09:00:00Z",
    });
    let decided = |status: &str, comment: Value| {
        let mut period = period_example.clone();
        period["status"] = json!(status);
        period["decided_by_member_id"] = json!("018f3b2a-0000-7000-8000-000000000005");
        period["decided_at"] = json!("2026-11-09T09:00:00Z");
        period["decision_comment"] = comment;
        period
    };

    vec![
        EventDescriptor {
            name: "extra_period.requested",
            version: 1,
            label: "Heures supplémentaires déclarées",
            subject_kind: "extra_period",
            payload_example: json!({ "extra_period": period_example }),
        },
        EventDescriptor {
            name: "extra_period.approved",
            version: 1,
            label: "Heures supplémentaires approuvées",
            subject_kind: "extra_period",
            payload_example: json!({
                "extra_period": decided("APPROVED", Value::Null),
                "from": "PENDING",
                "to": "APPROVED",
            }),
        },
        EventDescriptor {
            name: "extra_period.rejected",
            version: 1,
            label: "Heures supplémentaires refusées",
            subject_kind: "extra_period",
            payload_example: json!({
                "extra_period": decided("REJECTED", json!("Heures non validées par le chef d'équipe")),
                "from": "PENDING",
                "to": "REJECTED",
            }),
        },
        EventDescriptor {
            name: "extra_period.cancelled",
            version: 1,
            label: "Heures supplémentaires annulées",
            subject_kind: "extra_period",
            payload_example: json!({
                "extra_period": decided("CANCELLED", Value::Null),
                "from": "PENDING",
                "to": "CANCELLED",
            }),
        },
    ]
}
