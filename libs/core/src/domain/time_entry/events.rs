//! What field clocking publishes.
//!
//! Three business acts, no content or status events: a time entry is not
//! edited, it is started and stopped. Automations key on these to notify a
//! foreman, and M6 will read them to attribute cost to a job.

use events::{DomainEvent, EventDescriptor, EventSubject};
use serde_json::{Value, json};

use crate::{DayLog, TimeEntry};

pub struct TimeEntryStarted {
    pub entry: TimeEntry,
}

impl DomainEvent for TimeEntryStarted {
    fn name(&self) -> &'static str {
        "time_entry.started"
    }

    fn version(&self) -> u16 {
        1
    }

    fn subject(&self) -> EventSubject {
        EventSubject::new("time_entry", self.entry.id.0)
    }

    fn payload(&self) -> Value {
        json!({ "time_entry": entry_payload(&self.entry) })
    }
}

pub struct TimeEntryStopped {
    pub entry: TimeEntry,
}

impl DomainEvent for TimeEntryStopped {
    fn name(&self) -> &'static str {
        "time_entry.stopped"
    }

    fn version(&self) -> u16 {
        1
    }

    fn subject(&self) -> EventSubject {
        EventSubject::new("time_entry", self.entry.id.0)
    }

    fn payload(&self) -> Value {
        json!({
            "time_entry": entry_payload(&self.entry),
            "worked_minutes": self.entry.worked_minutes(),
        })
    }
}

/// A stretch of work declared after the fact, with no live start ever having
/// happened.
///
/// Not `TimeEntryStarted` followed by `TimeEntryStopped`: an automation that
/// notifies a foreman "so-and-so just started" would be lying, since the
/// work is already over by the time this fires. One event, carrying the whole
/// closed stretch, is the honest shape for something reported after it ended.
pub struct TimeEntryDeclared {
    pub entry: TimeEntry,
}

impl DomainEvent for TimeEntryDeclared {
    fn name(&self) -> &'static str {
        "time_entry.declared"
    }

    fn version(&self) -> u16 {
        1
    }

    fn subject(&self) -> EventSubject {
        EventSubject::new("time_entry", self.entry.id.0)
    }

    fn payload(&self) -> Value {
        json!({
            "time_entry": entry_payload(&self.entry),
            "worked_minutes": self.entry.worked_minutes(),
        })
    }
}

pub struct DayEnded {
    pub day_log: DayLog,
    /// How many running entries the declaration had to close. Zero is the
    /// normal case; anything else says the employee forgot to clock off.
    pub closed_entries: usize,
}

impl DomainEvent for DayEnded {
    fn name(&self) -> &'static str {
        "day.ended"
    }

    fn version(&self) -> u16 {
        1
    }

    fn subject(&self) -> EventSubject {
        EventSubject::new("day_log", self.day_log.id.0)
    }

    fn payload(&self) -> Value {
        json!({
            "day_log": {
                "id": self.day_log.id.0,
                "organization_id": self.day_log.organization_id.0,
                "employee_id": self.day_log.employee_id.0,
                "work_date": self.day_log.work_date,
                "ended_at": self.day_log.ended_at,
            },
            "closed_entries": self.closed_entries,
        })
    }
}

fn entry_payload(entry: &TimeEntry) -> Value {
    json!({
        "id": entry.id.0,
        "organization_id": entry.organization_id.0,
        "task_id": entry.task_id.0,
        "employee_id": entry.employee_id.0,
        "started_at": entry.started_at,
        "ended_at": entry.ended_at,
    })
}

pub fn descriptors() -> Vec<EventDescriptor> {
    let entry_example = json!({
        "id": "018f3b2a-0000-7000-8000-000000000001",
        "organization_id": "018f3b2a-0000-7000-8000-000000000002",
        "task_id": "018f3b2a-0000-7000-8000-000000000003",
        "employee_id": "018f3b2a-0000-7000-8000-000000000004",
        "started_at": "2026-08-18T07:30:00Z",
        "ended_at": Value::Null,
    });

    let mut closed_example = entry_example.clone();
    closed_example["ended_at"] = json!("2026-08-18T11:45:00Z");

    vec![
        EventDescriptor {
            name: "time_entry.started",
            version: 1,
            label: "Pointage démarré",
            subject_kind: "time_entry",
            payload_example: json!({ "time_entry": entry_example }),
        },
        EventDescriptor {
            name: "time_entry.stopped",
            version: 1,
            label: "Pointage clôturé",
            subject_kind: "time_entry",
            payload_example: json!({
                "time_entry": closed_example,
                "worked_minutes": 255,
            }),
        },
        EventDescriptor {
            name: "time_entry.declared",
            version: 1,
            label: "Temps déclaré après coup",
            subject_kind: "time_entry",
            payload_example: json!({
                "time_entry": closed_example,
                "worked_minutes": 255,
            }),
        },
        EventDescriptor {
            name: "day.ended",
            version: 1,
            label: "Journée terminée",
            subject_kind: "day_log",
            payload_example: json!({
                "day_log": {
                    "id": "018f3b2a-0000-7000-8000-000000000005",
                    "organization_id": "018f3b2a-0000-7000-8000-000000000002",
                    "employee_id": "018f3b2a-0000-7000-8000-000000000004",
                    "work_date": "2026-08-18",
                    "ended_at": "2026-08-18T16:00:00Z",
                },
                "closed_entries": 1,
            }),
        },
    ]
}

/// Every event this module can construct, for the catalogue drift check.
#[cfg(test)]
pub fn emitted_events() -> Vec<(&'static str, u16)> {
    vec![
        ("time_entry.started", 1),
        ("time_entry.stopped", 1),
        ("time_entry.declared", 1),
        ("day.ended", 1),
    ]
}
