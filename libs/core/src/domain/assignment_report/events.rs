//! What the correction loop publishes.
//!
//! Three business acts: a report is filed, then it is either applied or
//! dismissed. Amending or withdrawing a still-pending report emits nothing —
//! neither is a decision anyone downstream needs to react to, unlike the
//! moment a report is first filed or finally arbitrated.

use events::{DomainEvent, EventDescriptor, EventSubject};
use serde_json::{Value, json};

use crate::AssignmentReport;

pub struct AssignmentReportReported {
    pub report: AssignmentReport,
}

impl DomainEvent for AssignmentReportReported {
    fn name(&self) -> &'static str {
        "assignment_report.reported"
    }

    fn version(&self) -> u16 {
        1
    }

    fn subject(&self) -> EventSubject {
        EventSubject::new("assignment_report", self.report.id.0)
    }

    fn payload(&self) -> Value {
        json!({ "assignment_report": report_payload(&self.report) })
    }
}

pub struct AssignmentReportApplied {
    pub report: AssignmentReport,
}

impl DomainEvent for AssignmentReportApplied {
    fn name(&self) -> &'static str {
        "assignment_report.applied"
    }

    fn version(&self) -> u16 {
        1
    }

    fn subject(&self) -> EventSubject {
        EventSubject::new("assignment_report", self.report.id.0)
    }

    fn payload(&self) -> Value {
        json!({ "assignment_report": report_payload(&self.report) })
    }
}

pub struct AssignmentReportDismissed {
    pub report: AssignmentReport,
}

impl DomainEvent for AssignmentReportDismissed {
    fn name(&self) -> &'static str {
        "assignment_report.dismissed"
    }

    fn version(&self) -> u16 {
        1
    }

    fn subject(&self) -> EventSubject {
        EventSubject::new("assignment_report", self.report.id.0)
    }

    fn payload(&self) -> Value {
        json!({ "assignment_report": report_payload(&self.report) })
    }
}

fn report_payload(report: &AssignmentReport) -> Value {
    json!({
        "id": report.id.0,
        "organization_id": report.organization_id.0,
        "task_assignment_id": report.task_assignment_id.0,
        "reported_minutes": report.reported_minutes,
        "reported_by": report.reported_by.0,
        "resolution": report.resolution.as_str(),
        "resolved_by": report.resolved_by.map(|id| id.0),
        "resolved_at": report.resolved_at,
    })
}

pub fn descriptors() -> Vec<EventDescriptor> {
    let reported_example = json!({
        "id": "018f3b2a-0000-7000-8000-000000000010",
        "organization_id": "018f3b2a-0000-7000-8000-000000000002",
        "task_assignment_id": "018f3b2a-0000-7000-8000-000000000011",
        "reported_minutes": 300,
        "reported_by": "018f3b2a-0000-7000-8000-000000000012",
        "resolution": "PENDING",
        "resolved_by": Value::Null,
        "resolved_at": Value::Null,
    });

    let mut applied_example = reported_example.clone();
    applied_example["resolution"] = json!("APPLIED");
    applied_example["resolved_by"] = json!("018f3b2a-0000-7000-8000-000000000013");
    applied_example["resolved_at"] = json!("2026-08-22T16:00:00Z");

    let mut dismissed_example = applied_example.clone();
    dismissed_example["resolution"] = json!("DISMISSED");

    vec![
        EventDescriptor {
            name: "assignment_report.reported",
            version: 1,
            label: "Écart déclaré",
            subject_kind: "assignment_report",
            payload_example: json!({ "assignment_report": reported_example }),
        },
        EventDescriptor {
            name: "assignment_report.applied",
            version: 1,
            label: "Écart appliqué au planning",
            subject_kind: "assignment_report",
            payload_example: json!({ "assignment_report": applied_example }),
        },
        EventDescriptor {
            name: "assignment_report.dismissed",
            version: 1,
            label: "Écart rejeté",
            subject_kind: "assignment_report",
            payload_example: json!({ "assignment_report": dismissed_example }),
        },
    ]
}

/// Every event this module can construct, for the catalogue drift check.
#[cfg(test)]
pub fn emitted_events() -> Vec<(&'static str, u16)> {
    vec![
        ("assignment_report.reported", 1),
        ("assignment_report.applied", 1),
        ("assignment_report.dismissed", 1),
    ]
}
