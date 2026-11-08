//! What the customer aggregate publishes.
//!
//! One event today: [`CustomerCreated`]. A later change to a customer stays
//! silent: what automations ask for is "a new customer arrived", and a
//! prospect turning into a client already shows through `deal.stage_changed`.

use events::{DomainEvent, EventDescriptor, EventSubject};
use serde_json::{Value, json};

use crate::Customer;

pub struct CustomerCreated {
    pub customer: Customer,
}

impl DomainEvent for CustomerCreated {
    fn name(&self) -> &'static str {
        "customer.created"
    }

    fn version(&self) -> u16 {
        1
    }

    fn subject(&self) -> EventSubject {
        EventSubject::new("customer", self.customer.id.0)
    }

    fn payload(&self) -> Value {
        json!({ "customer": customer_payload(&self.customer) })
    }
}

/// The serialized domain model, never the database row: renaming a column must
/// not reach a subscriber's automation.
fn customer_payload(customer: &Customer) -> Value {
    json!({
        "id": customer.id.0,
        "organization_id": customer.organization_id.0,
        "status": customer.status,
        "kind": customer.profile.kind,
        "name": customer.name,
        "email": customer.email,
        "phone": customer.phone,
        "registration_number": customer.registration_number,
        "vat_number": customer.profile.vat_number,
        "owner_member_id": customer.profile.owner_member_id.map(|id| id.0),
        "lead_source": customer.profile.lead_source,
        "created_at": customer.created_at,
    })
}

/// Test-only, mirrors `quote::events::emitted_events`.
#[cfg(test)]
pub fn emitted_events() -> Vec<(&'static str, u16)> {
    vec![("customer.created", 1)]
}

pub fn descriptors() -> Vec<EventDescriptor> {
    vec![EventDescriptor {
        name: "customer.created",
        version: 1,
        label: "Client créé",
        subject_kind: "customer",
        payload_example: json!({
            "customer": {
                "id": "018f3b2a-0000-7000-8000-000000000003",
                "organization_id": "018f3b2a-0000-7000-8000-000000000002",
                "status": "PROSPECT",
                "kind": "PROFESSIONAL",
                "name": "Boulangerie Lemoine",
                "email": "contact@lemoine.fr",
                "phone": "0102030405",
                "registration_number": "12345678901234",
                "vat_number": null,
                "owner_member_id": null,
                "lead_source": "bouche_a_oreille",
                "created_at": "2026-08-08T09:00:00Z",
            }
        }),
    }]
}
