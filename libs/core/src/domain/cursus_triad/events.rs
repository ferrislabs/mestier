use events::{DomainEvent, EventDescriptor, EventSubject};
use serde_json::{Value, json};

use crate::{Closure, Contract, Triad};

pub struct TriadCreated<'a> {
    pub triad: &'a Triad,
}

impl DomainEvent for TriadCreated<'_> {
    fn name(&self) -> &'static str {
        "cursus.triad.created"
    }

    fn version(&self) -> u16 {
        1
    }

    fn subject(&self) -> EventSubject {
        EventSubject::new("triad", self.triad.id.0)
    }

    fn payload(&self) -> Value {
        json!({
            "triad_id": self.triad.id.0,
            "organization_id": self.triad.organization_id.0,
            "promotion_id": self.triad.promotion_id.0,
            "apprentice_id": self.triad.apprentice_id.0,
            "pedagogic_tutor_member_id": self.triad.pedagogic_tutor_member_id.map(|id| id.0),
        })
    }
}

pub struct TriadClosed<'a> {
    pub triad: &'a Triad,
    pub closure: &'a Closure,
}

impl DomainEvent for TriadClosed<'_> {
    fn name(&self) -> &'static str {
        "cursus.triad.closed"
    }

    fn version(&self) -> u16 {
        1
    }

    fn subject(&self) -> EventSubject {
        EventSubject::new("triad", self.triad.id.0)
    }

    fn payload(&self) -> Value {
        json!({
            "triad_id": self.triad.id.0,
            "organization_id": self.triad.organization_id.0,
            "closed_at": self.closure.closed_at,
            "closure_reason_id": self.closure.reason_id.0,
            "closure_note": self.closure.note,
        })
    }
}

pub struct ContractCreated<'a> {
    pub contract: &'a Contract,
}

impl DomainEvent for ContractCreated<'_> {
    fn name(&self) -> &'static str {
        "cursus.contract.created"
    }

    fn version(&self) -> u16 {
        1
    }

    fn subject(&self) -> EventSubject {
        EventSubject::new("contract", self.contract.id.0)
    }

    fn payload(&self) -> Value {
        json!({
            "contract_id": self.contract.id.0,
            "triad_id": self.contract.triad_id.0,
            "organization_id": self.contract.organization_id.0,
            "host_company_id": self.contract.host_company_id.0,
            "kind": self.contract.kind,
            "start_date": self.contract.period.start(),
            "end_date": self.contract.period.end(),
        })
    }
}

pub struct ContractClosed<'a> {
    pub contract: &'a Contract,
    pub closure: &'a Closure,
}

impl DomainEvent for ContractClosed<'_> {
    fn name(&self) -> &'static str {
        "cursus.contract.closed"
    }

    fn version(&self) -> u16 {
        1
    }

    fn subject(&self) -> EventSubject {
        EventSubject::new("contract", self.contract.id.0)
    }

    fn payload(&self) -> Value {
        json!({
            "contract_id": self.contract.id.0,
            "triad_id": self.contract.triad_id.0,
            "organization_id": self.contract.organization_id.0,
            "closed_at": self.closure.closed_at,
            "closure_reason_id": self.closure.reason_id.0,
            "closure_note": self.closure.note,
        })
    }
}

#[cfg(test)]
pub fn emitted_events() -> Vec<(&'static str, u16)> {
    vec![
        ("cursus.triad.created", 1),
        ("cursus.triad.closed", 1),
        ("cursus.contract.created", 1),
        ("cursus.contract.closed", 1),
    ]
}

pub fn descriptors() -> Vec<EventDescriptor> {
    vec![
        EventDescriptor {
            name: "cursus.triad.created",
            version: 1,
            label: "Trinôme créé",
            subject_kind: "triad",
            payload_example: json!({
                "triad_id": "018f3b2a-0000-7000-8000-000000000001",
                "organization_id": "018f3b2a-0000-7000-8000-000000000002",
                "promotion_id": "018f3b2a-0000-7000-8000-000000000003",
                "apprentice_id": "018f3b2a-0000-7000-8000-000000000004",
                "pedagogic_tutor_member_id": "018f3b2a-0000-7000-8000-000000000005",
            }),
        },
        EventDescriptor {
            name: "cursus.triad.closed",
            version: 1,
            label: "Trinôme clos",
            subject_kind: "triad",
            payload_example: json!({
                "triad_id": "018f3b2a-0000-7000-8000-000000000001",
                "organization_id": "018f3b2a-0000-7000-8000-000000000002",
                "closed_at": "2027-06-30",
                "closure_reason_id": "018f3b2a-0000-7000-8000-000000000006",
                "closure_note": "Fin de formation",
            }),
        },
        EventDescriptor {
            name: "cursus.contract.created",
            version: 1,
            label: "Contrat créé",
            subject_kind: "contract",
            payload_example: json!({
                "contract_id": "018f3b2a-0000-7000-8000-000000000007",
                "triad_id": "018f3b2a-0000-7000-8000-000000000001",
                "organization_id": "018f3b2a-0000-7000-8000-000000000002",
                "host_company_id": "018f3b2a-0000-7000-8000-000000000008",
                "kind": "APPRENTICESHIP",
                "start_date": "2026-09-01",
                "end_date": "2028-08-31",
            }),
        },
        EventDescriptor {
            name: "cursus.contract.closed",
            version: 1,
            label: "Contrat clos",
            subject_kind: "contract",
            payload_example: json!({
                "contract_id": "018f3b2a-0000-7000-8000-000000000007",
                "triad_id": "018f3b2a-0000-7000-8000-000000000001",
                "organization_id": "018f3b2a-0000-7000-8000-000000000002",
                "closed_at": "2027-06-30",
                "closure_reason_id": "018f3b2a-0000-7000-8000-000000000006",
                "closure_note": "Rupture à l'amiable",
            }),
        },
    ]
}
