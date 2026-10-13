//! What the intervention loop publishes: an intervention is proposed by the
//! field, then validated or rejected. A manager's direct entry is validated
//! at once and emits only `validated`.

use events::{DomainEvent, EventDescriptor, EventSubject};
use serde_json::{Value, json};

use crate::Intervention;

pub struct InterventionProposed {
    pub intervention: Intervention,
}

impl DomainEvent for InterventionProposed {
    fn name(&self) -> &'static str {
        "maintenance_intervention.proposed"
    }

    fn version(&self) -> u16 {
        1
    }

    fn subject(&self) -> EventSubject {
        EventSubject::new("maintenance_intervention", self.intervention.id.0)
    }

    fn payload(&self) -> Value {
        json!({ "maintenance_intervention": intervention_payload(&self.intervention) })
    }
}

pub struct InterventionValidated {
    pub intervention: Intervention,
}

impl DomainEvent for InterventionValidated {
    fn name(&self) -> &'static str {
        "maintenance_intervention.validated"
    }

    fn version(&self) -> u16 {
        1
    }

    fn subject(&self) -> EventSubject {
        EventSubject::new("maintenance_intervention", self.intervention.id.0)
    }

    fn payload(&self) -> Value {
        json!({ "maintenance_intervention": intervention_payload(&self.intervention) })
    }
}

pub struct InterventionRejected {
    pub intervention: Intervention,
}

impl DomainEvent for InterventionRejected {
    fn name(&self) -> &'static str {
        "maintenance_intervention.rejected"
    }

    fn version(&self) -> u16 {
        1
    }

    fn subject(&self) -> EventSubject {
        EventSubject::new("maintenance_intervention", self.intervention.id.0)
    }

    fn payload(&self) -> Value {
        json!({ "maintenance_intervention": intervention_payload(&self.intervention) })
    }
}

fn intervention_payload(intervention: &Intervention) -> Value {
    json!({
        "id": intervention.id.0,
        "organization_id": intervention.organization_id.0,
        "line_id": intervention.line_id.0,
        "done_on": intervention.done_on,
        "status": intervention.status.as_str(),
        "task_id": intervention.task_id.map(|id| id.0),
        "proposed_by": intervention.proposed_by.0,
        "resolved_by": intervention.resolved_by.map(|id| id.0),
        "resolved_at": intervention.resolved_at,
    })
}

pub fn descriptors() -> Vec<EventDescriptor> {
    let proposed_example = json!({
        "id": "018f3b2a-0000-7000-8000-000000000020",
        "organization_id": "018f3b2a-0000-7000-8000-000000000002",
        "line_id": "018f3b2a-0000-7000-8000-000000000021",
        "done_on": "2026-10-14",
        "status": "PROPOSED",
        "task_id": "018f3b2a-0000-7000-8000-000000000022",
        "proposed_by": "018f3b2a-0000-7000-8000-000000000012",
        "resolved_by": Value::Null,
        "resolved_at": Value::Null,
    });

    let mut validated_example = proposed_example.clone();
    validated_example["status"] = json!("VALIDATED");
    validated_example["resolved_by"] = json!("018f3b2a-0000-7000-8000-000000000013");
    validated_example["resolved_at"] = json!("2026-10-15T08:00:00Z");

    let mut rejected_example = validated_example.clone();
    rejected_example["status"] = json!("REJECTED");

    vec![
        EventDescriptor {
            name: "maintenance_intervention.proposed",
            version: 1,
            label: "Intervention d'entretien déclarée",
            subject_kind: "maintenance_intervention",
            payload_example: json!({ "maintenance_intervention": proposed_example }),
        },
        EventDescriptor {
            name: "maintenance_intervention.validated",
            version: 1,
            label: "Intervention d'entretien validée",
            subject_kind: "maintenance_intervention",
            payload_example: json!({ "maintenance_intervention": validated_example }),
        },
        EventDescriptor {
            name: "maintenance_intervention.rejected",
            version: 1,
            label: "Intervention d'entretien rejetée",
            subject_kind: "maintenance_intervention",
            payload_example: json!({ "maintenance_intervention": rejected_example }),
        },
    ]
}

/// Every event this module can construct, for the catalogue drift check.
#[cfg(test)]
pub fn emitted_events() -> Vec<(&'static str, u16)> {
    vec![
        ("maintenance_intervention.proposed", 1),
        ("maintenance_intervention.validated", 1),
        ("maintenance_intervention.rejected", 1),
    ]
}
