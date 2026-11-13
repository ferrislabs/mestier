use events::{DomainEvent, EventDescriptor, EventSubject};
use serde_json::{Value, json};

use crate::SchoolTutor;

pub struct SchoolTutorDeclared<'a> {
    pub tutor: &'a SchoolTutor,
}

impl DomainEvent for SchoolTutorDeclared<'_> {
    fn name(&self) -> &'static str {
        "cursus.school_tutor.declared"
    }

    fn version(&self) -> u16 {
        1
    }

    fn subject(&self) -> EventSubject {
        EventSubject::new("school_tutor", self.tutor.id.0)
    }

    fn payload(&self) -> Value {
        json!({
            "school_tutor_id": self.tutor.id.0,
            "organization_id": self.tutor.organization_id.0,
            "member_id": self.tutor.member_id.0,
            "job_title": self.tutor.job_title,
        })
    }
}

pub struct SchoolTutorArchived<'a> {
    pub tutor: &'a SchoolTutor,
}

impl DomainEvent for SchoolTutorArchived<'_> {
    fn name(&self) -> &'static str {
        "cursus.school_tutor.archived"
    }

    fn version(&self) -> u16 {
        1
    }

    fn subject(&self) -> EventSubject {
        EventSubject::new("school_tutor", self.tutor.id.0)
    }

    fn payload(&self) -> Value {
        json!({
            "school_tutor_id": self.tutor.id.0,
            "organization_id": self.tutor.organization_id.0,
            "member_id": self.tutor.member_id.0,
            "archived_at": self.tutor.archived_at,
        })
    }
}

#[cfg(test)]
pub fn emitted_events() -> Vec<(&'static str, u16)> {
    vec![
        ("cursus.school_tutor.declared", 1),
        ("cursus.school_tutor.archived", 1),
    ]
}

pub fn descriptors() -> Vec<EventDescriptor> {
    vec![
        EventDescriptor {
            name: "cursus.school_tutor.declared",
            version: 1,
            label: "Tuteur école déclaré",
            subject_kind: "school_tutor",
            payload_example: json!({
                "school_tutor_id": "018f3b2a-0000-7000-8000-000000000001",
                "organization_id": "018f3b2a-0000-7000-8000-000000000002",
                "member_id": "018f3b2a-0000-7000-8000-000000000003",
                "job_title": "Formateur référent",
            }),
        },
        EventDescriptor {
            name: "cursus.school_tutor.archived",
            version: 1,
            label: "Tuteur école archivé",
            subject_kind: "school_tutor",
            payload_example: json!({
                "school_tutor_id": "018f3b2a-0000-7000-8000-000000000001",
                "organization_id": "018f3b2a-0000-7000-8000-000000000002",
                "member_id": "018f3b2a-0000-7000-8000-000000000003",
                "archived_at": "2026-11-13T09:00:00Z",
            }),
        },
    ]
}
