//! What the deal aggregate publishes.
//!
//! [`DealCreated`] when a deal opens, [`DealStageChanged`] whenever it moves,
//! including the move to `LOST` that archiving a customer applies to its open
//! deals. A move to the stage a deal is already in publishes nothing.

use events::{DomainEvent, EventDescriptor, EventSubject};
use serde_json::{Value, json};

use crate::{Deal, DealStage, DealStageName};

pub struct DealCreated {
    pub deal: Deal,
}

impl DomainEvent for DealCreated {
    fn name(&self) -> &'static str {
        "deal.created"
    }

    fn version(&self) -> u16 {
        1
    }

    fn subject(&self) -> EventSubject {
        EventSubject::new("deal", self.deal.id.0)
    }

    fn payload(&self) -> Value {
        json!({ "deal": deal_payload(&self.deal) })
    }
}

pub struct DealStageChanged {
    pub deal: Deal,
    pub from: DealStageName,
}

impl DomainEvent for DealStageChanged {
    fn name(&self) -> &'static str {
        "deal.stage_changed"
    }

    fn version(&self) -> u16 {
        1
    }

    fn subject(&self) -> EventSubject {
        EventSubject::new("deal", self.deal.id.0)
    }

    fn payload(&self) -> Value {
        json!({
            "deal": deal_payload(&self.deal),
            "from": self.from.as_str(),
            "to": self.deal.stage.name().as_str(),
        })
    }
}

/// The serialized domain model, never the database row: renaming a column must
/// not reach a subscriber's automation.
fn deal_payload(deal: &Deal) -> Value {
    let lost_reason = match &deal.stage {
        DealStage::Lost(reason) => Some(reason.as_str()),
        _ => None,
    };

    json!({
        "id": deal.id.0,
        "organization_id": deal.organization_id.0,
        "customer_id": deal.customer_id.0,
        "title": deal.title.as_str(),
        "amount_cents": deal.amount.get(),
        "stage": deal.stage.name().as_str(),
        "lost_reason": lost_reason,
        "expected_close_date": deal.expected_close_date,
        "owner_member_id": deal.owner_member_id.map(|id| id.0),
        "created_at": deal.created_at,
        "stage_changed_at": deal.stage_changed_at,
        "closed_at": deal.closed_at,
    })
}

/// Test-only, mirrors `quote::events::emitted_events`.
#[cfg(test)]
pub fn emitted_events() -> Vec<(&'static str, u16)> {
    vec![("deal.created", 1), ("deal.stage_changed", 1)]
}

pub fn descriptors() -> Vec<EventDescriptor> {
    let deal_example = json!({
        "id": "018f3b2a-0000-7000-8000-000000000006",
        "organization_id": "018f3b2a-0000-7000-8000-000000000002",
        "customer_id": "018f3b2a-0000-7000-8000-000000000003",
        "title": "Rénovation du fournil",
        "amount_cents": 1_250_000,
        "stage": "NEW",
        "lost_reason": null,
        "expected_close_date": "2026-10-15",
        "owner_member_id": null,
        "created_at": "2026-08-08T09:00:00Z",
        "stage_changed_at": "2026-08-08T09:00:00Z",
        "closed_at": null,
    });

    vec![
        EventDescriptor {
            name: "deal.created",
            version: 1,
            label: "Opportunité créée",
            subject_kind: "deal",
            payload_example: json!({ "deal": deal_example }),
        },
        EventDescriptor {
            name: "deal.stage_changed",
            version: 1,
            label: "Opportunité déplacée dans le pipeline",
            subject_kind: "deal",
            payload_example: json!({
                "deal": deal_example,
                "from": "NEW",
                "to": "VISIT",
            }),
        },
    ]
}
