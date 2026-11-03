use events::{DomainEvent, EventDescriptor, EventSubject};
use serde_json::{Value, json};

use crate::{Ticket, TicketStatus};

pub struct TicketCreated {
    pub ticket: Ticket,
}

impl DomainEvent for TicketCreated {
    fn name(&self) -> &'static str {
        "ticket.created"
    }

    fn version(&self) -> u16 {
        1
    }

    fn subject(&self) -> EventSubject {
        EventSubject::new("ticket", self.ticket.id.0)
    }

    fn payload(&self) -> Value {
        json!({ "ticket": ticket_payload(&self.ticket) })
    }
}

pub struct TicketTransitioned {
    pub ticket: Ticket,
    pub from: TicketStatus,
    pub to: TicketStatus,
}

impl DomainEvent for TicketTransitioned {
    fn name(&self) -> &'static str {
        "ticket.transitioned"
    }

    fn version(&self) -> u16 {
        1
    }

    fn subject(&self) -> EventSubject {
        EventSubject::new("ticket", self.ticket.id.0)
    }

    fn payload(&self) -> Value {
        json!({
            "ticket": ticket_payload(&self.ticket),
            "from": self.from.as_str(),
            "to": self.to.as_str(),
        })
    }
}

pub fn ticket_payload(ticket: &Ticket) -> Value {
    let badges: Vec<Value> = ticket
        .badges
        .iter()
        .map(|badge| json!({ "label": badge.label(), "tone": badge.tone().as_str() }))
        .collect();

    json!({
        "id": ticket.id.0,
        "organization_id": ticket.organization_id.0,
        "number": ticket.number.0,
        "subject": ticket.subject,
        "body": ticket.body,
        "status": ticket.status.as_str(),
        "assignee_member_id": ticket.assignee.map(|member| member.0),
        "assigned_at": ticket.assigned_at,
        "custom_fields": ticket.custom_fields.as_map(),
        "badges": badges,
        "external_id": ticket.external_id,
        "created_at": ticket.created_at,
        "updated_at": ticket.updated_at,
    })
}

pub fn descriptors() -> Vec<EventDescriptor> {
    let created_example = json!({
        "id": "018f3b2a-0000-7000-8000-000000000030",
        "organization_id": "018f3b2a-0000-7000-8000-000000000002",
        "number": 42,
        "subject": "Impossible de déployer en production",
        "body": "Le déploiement dpl_8f2a91 échoue depuis ce matin avec une erreur 502.",
        "status": "new",
        "assignee_member_id": Value::Null,
        "assigned_at": Value::Null,
        "custom_fields": {
            "deployment_id": "dpl_8f2a91",
            "subscription": "enterprise",
            "team": "tech",
            "priority": "high",
        },
        "badges": [],
        "external_id": "ZD-48213",
        "created_at": "2026-10-15T07:45:00Z",
        "updated_at": "2026-10-15T07:45:00Z",
    });

    let mut transitioned_example = created_example.clone();
    transitioned_example["status"] = json!("in_progress");
    transitioned_example["assignee_member_id"] = json!("018f3b2a-0000-7000-8000-000000000012");
    transitioned_example["assigned_at"] = json!("2026-10-15T08:00:00Z");
    transitioned_example["badges"] = json!([{ "label": "Bloquant", "tone": "error" }]);
    transitioned_example["updated_at"] = json!("2026-10-15T09:10:00Z");

    vec![
        EventDescriptor {
            name: "ticket.created",
            version: 1,
            label: "Ticket de support reçu",
            subject_kind: "ticket",
            payload_example: json!({ "ticket": created_example }),
        },
        EventDescriptor {
            name: "ticket.transitioned",
            version: 1,
            label: "Statut d'un ticket de support modifié",
            subject_kind: "ticket",
            payload_example: json!({
                "ticket": transitioned_example,
                "from": "waiting_customer",
                "to": "in_progress",
            }),
        },
    ]
}

#[cfg(test)]
pub fn emitted_events() -> Vec<(&'static str, u16)> {
    vec![("ticket.created", 1), ("ticket.transitioned", 1)]
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use chrono::{DateTime, Utc};
    use events::{DomainEvent, EventSubject};
    use serde_json::{Value, json};
    use uuid::Uuid;

    use super::*;
    use crate::{
        Badge, BadgeTone, CustomFields, MemberId, OrganizationId, Ticket, TicketId, TicketNumber,
        TicketStatus,
    };

    fn instant(text: &str) -> DateTime<Utc> {
        text.parse().unwrap()
    }

    fn ticket() -> Ticket {
        Ticket {
            id: TicketId(Uuid::from_u128(0x30)),
            organization_id: OrganizationId(Uuid::from_u128(0x02)),
            number: TicketNumber(42),
            subject: "Impossible de déployer".to_owned(),
            body: "Le déploiement échoue avec une erreur 502.".to_owned(),
            status: TicketStatus::New,
            assignee: None,
            assigned_at: None,
            custom_fields: CustomFields::try_from(json!({
                "deployment_id": "dpl_8f2a91",
                "subscription": "enterprise",
            }))
            .unwrap(),
            badges: vec![Badge::new("Bloquant", BadgeTone::Error).unwrap()],
            external_id: Some("ZD-48213".to_owned()),
            created_at: instant("2026-10-15T07:45:00Z"),
            updated_at: instant("2026-10-15T07:45:00Z"),
        }
    }

    fn keys(value: &Value) -> BTreeSet<&str> {
        value
            .as_object()
            .unwrap()
            .keys()
            .map(String::as_str)
            .collect()
    }

    #[test]
    fn ticket_created_payload_nests_the_ticket_with_custom_fields() {
        let ticket = ticket();
        let event = TicketCreated {
            ticket: ticket.clone(),
        };

        assert_eq!(event.name(), "ticket.created");
        assert_eq!(event.version(), 1);
        assert_eq!(event.subject(), EventSubject::new("ticket", ticket.id.0));
        assert_eq!(
            event.payload(),
            json!({
                "ticket": {
                    "id": "00000000-0000-0000-0000-000000000030",
                    "organization_id": "00000000-0000-0000-0000-000000000002",
                    "number": 42,
                    "subject": "Impossible de déployer",
                    "body": "Le déploiement échoue avec une erreur 502.",
                    "status": "new",
                    "assignee_member_id": null,
                    "assigned_at": null,
                    "custom_fields": {
                        "deployment_id": "dpl_8f2a91",
                        "subscription": "enterprise",
                    },
                    "badges": [{ "label": "Bloquant", "tone": "error" }],
                    "external_id": "ZD-48213",
                    "created_at": "2026-10-15T07:45:00Z",
                    "updated_at": "2026-10-15T07:45:00Z",
                }
            })
        );
    }

    #[test]
    fn ticket_transitioned_payload_carries_from_and_to() {
        let mut ticket = ticket();
        ticket.status = TicketStatus::InProgress;
        ticket.assignee = Some(MemberId(Uuid::from_u128(0x12)));
        ticket.assigned_at = Some(instant("2026-10-15T08:00:00Z"));
        ticket.external_id = None;
        let event = TicketTransitioned {
            ticket: ticket.clone(),
            from: TicketStatus::WaitingCustomer,
            to: TicketStatus::InProgress,
        };

        let payload = event.payload();

        assert_eq!(event.name(), "ticket.transitioned");
        assert_eq!(event.version(), 1);
        assert_eq!(event.subject(), EventSubject::new("ticket", ticket.id.0));
        assert_eq!(payload["from"], json!("waiting_customer"));
        assert_eq!(payload["to"], json!("in_progress"));
        assert_eq!(payload["ticket"], ticket_payload(&ticket));
        assert_eq!(payload["ticket"]["status"], json!("in_progress"));
        assert_eq!(
            payload["ticket"]["assignee_member_id"],
            json!("00000000-0000-0000-0000-000000000012")
        );
        assert_eq!(
            payload["ticket"]["assigned_at"],
            json!("2026-10-15T08:00:00Z")
        );
        assert_eq!(payload["ticket"]["external_id"], Value::Null);
        assert_eq!(keys(&payload), BTreeSet::from(["from", "to", "ticket"]));
    }

    #[test]
    fn every_descriptor_has_a_non_null_example_shaped_like_the_payload() {
        let ticket = ticket();
        let created = TicketCreated {
            ticket: ticket.clone(),
        }
        .payload();
        let transitioned = TicketTransitioned {
            ticket: ticket.clone(),
            from: TicketStatus::New,
            to: TicketStatus::Resolved,
        }
        .payload();
        let descriptors = descriptors();

        assert_eq!(
            descriptors
                .iter()
                .map(|descriptor| (descriptor.name, descriptor.version, descriptor.subject_kind))
                .collect::<Vec<_>>(),
            vec![
                ("ticket.created", 1, "ticket"),
                ("ticket.transitioned", 1, "ticket")
            ]
        );
        for descriptor in &descriptors {
            let actual = match descriptor.name {
                "ticket.created" => &created,
                _ => &transitioned,
            };
            let example = &descriptor.payload_example;

            assert!(!example.is_null(), "{}", descriptor.name);
            assert_eq!(keys(example), keys(actual), "{}", descriptor.name);
            assert_eq!(
                keys(&example["ticket"]),
                keys(&actual["ticket"]),
                "{}",
                descriptor.name
            );
            assert_eq!(
                example["ticket"]["custom_fields"],
                json!({
                    "deployment_id": "dpl_8f2a91",
                    "subscription": "enterprise",
                    "team": "tech",
                    "priority": "high",
                }),
                "{}",
                descriptor.name
            );
            for badge in example["ticket"]["badges"].as_array().unwrap() {
                assert_eq!(keys(badge), BTreeSet::from(["label", "tone"]));
            }
        }
        assert_eq!(descriptors[0].label, "Ticket de support reçu");
        assert_eq!(
            descriptors[1].label,
            "Statut d'un ticket de support modifié"
        );
        assert_eq!(
            descriptors[1].payload_example["from"],
            json!("waiting_customer")
        );
        assert_eq!(descriptors[1].payload_example["to"], json!("in_progress"));
        assert_eq!(
            descriptors[1].payload_example["ticket"]["status"],
            json!("in_progress")
        );
        assert_eq!(
            emitted_events(),
            vec![("ticket.created", 1), ("ticket.transitioned", 1)]
        );
    }
}
