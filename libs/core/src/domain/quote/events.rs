//! What the quote aggregate publishes.
//!
//! Three categories, and they never overlap. A write touching two of them
//! emits two events, each restricted to its own perimeter:
//!
//! - **Content** — [`QuoteUpdated`], carrying only the fields that changed.
//! - **Status transition** — one named event per transition. Never reported as
//!   a content change.
//! - **Business act** — neither of the above. Quotes have none yet.

use events::{DomainEvent, EventDescriptor, EventSubject};
use serde_json::{Value, json};

use crate::{Quote, QuoteId, QuoteStatus};

pub struct QuoteCreated {
    pub quote: Quote,
}

impl DomainEvent for QuoteCreated {
    fn name(&self) -> &'static str {
        "quote.created"
    }

    fn version(&self) -> u16 {
        1
    }

    fn subject(&self) -> EventSubject {
        EventSubject::new("quote", self.quote.id.0)
    }

    fn payload(&self) -> Value {
        json!({ "quote": quote_payload(&self.quote) })
    }
}

pub struct QuoteUpdated {
    pub quote: Quote,
    pub changed_fields: Vec<&'static str>,
    pub previous: Value,
}

impl DomainEvent for QuoteUpdated {
    fn name(&self) -> &'static str {
        "quote.updated"
    }

    fn version(&self) -> u16 {
        1
    }

    fn subject(&self) -> EventSubject {
        EventSubject::new("quote", self.quote.id.0)
    }

    fn payload(&self) -> Value {
        json!({
            "quote": quote_payload(&self.quote),
            "changed_fields": self.changed_fields,
            "previous": self.previous,
        })
    }
}

pub struct QuoteDeleted {
    pub quote_id: QuoteId,
}

impl DomainEvent for QuoteDeleted {
    fn name(&self) -> &'static str {
        "quote.deleted"
    }

    fn version(&self) -> u16 {
        1
    }

    fn subject(&self) -> EventSubject {
        EventSubject::new("quote", self.quote_id.0)
    }

    fn payload(&self) -> Value {
        json!({ "quote_id": self.quote_id.0 })
    }
}

/// One transition of [`QuoteStatus`], named after where it lands.
pub struct QuoteTransitioned {
    pub quote: Quote,
    pub from: QuoteStatus,
}

impl QuoteTransitioned {
    /// `None` for a transition the product has no name for — today, anything
    /// landing back on `Draft`. A transition without a name is not emitted
    /// rather than emitted under a vague one.
    pub fn event_name(to: QuoteStatus) -> Option<&'static str> {
        match to {
            QuoteStatus::Sent => Some("quote.sent"),
            QuoteStatus::Accepted => Some("quote.accepted"),
            QuoteStatus::Declined => Some("quote.declined"),
            QuoteStatus::Cancelled => Some("quote.cancelled"),
            QuoteStatus::Draft => None,
        }
    }
}

impl DomainEvent for QuoteTransitioned {
    fn name(&self) -> &'static str {
        Self::event_name(self.quote.status).expect("a nameless transition is never emitted")
    }

    fn version(&self) -> u16 {
        1
    }

    fn subject(&self) -> EventSubject {
        EventSubject::new("quote", self.quote.id.0)
    }

    fn payload(&self) -> Value {
        json!({
            "quote": quote_payload(&self.quote),
            "from": self.from.as_str(),
            "to": self.quote.status.as_str(),
        })
    }
}

/// The serialized domain model, never the database row: renaming a column must
/// not reach a subscriber's automation.
fn quote_payload(quote: &Quote) -> Value {
    json!({
        "id": quote.id.0,
        "organization_id": quote.organization_id.0,
        "reference": quote.reference,
        "title": quote.title,
        "customer_id": quote.customer_id.0,
        "customer_context_id": quote.customer_context_id.0,
        "status": quote.status.as_str(),
        "total_cents": quote.total_cents,
        "created_at": quote.created_at,
        "updated_at": quote.updated_at,
    })
}

/// Every event this module can construct, as `(name, version)`.
///
/// Test-only: it exists so the drift check in `domain::automation::catalogue`
/// has something to compare the descriptors against. Kept beside the types
/// rather than derived by reflection — the list is short, and the transitions
/// come from the same exhaustive match the emission uses.
#[cfg(test)]
pub fn emitted_events() -> Vec<(&'static str, u16)> {
    let mut emitted = vec![
        ("quote.created", 1),
        ("quote.updated", 1),
        ("quote.deleted", 1),
    ];

    emitted.extend(
        QuoteStatus::ALL
            .into_iter()
            .filter_map(QuoteTransitioned::event_name)
            .map(|name| (name, 1)),
    );

    emitted
}

pub fn descriptors() -> Vec<EventDescriptor> {
    let quote_example = json!({
        "id": "018f3b2a-0000-7000-8000-000000000001",
        "organization_id": "018f3b2a-0000-7000-8000-000000000002",
        "reference": "DEV-2026-0001",
        "title": "Rénovation cuisine",
        "customer_id": "018f3b2a-0000-7000-8000-000000000003",
        "customer_context_id": "018f3b2a-0000-7000-8000-000000000004",
        "status": "DRAFT",
        "total_cents": 550_000,
        "created_at": "2026-08-08T09:00:00Z",
        "updated_at": "2026-08-08T09:00:00Z",
    });

    let mut descriptors = vec![
        EventDescriptor {
            name: "quote.created",
            version: 1,
            label: "Devis créé",
            subject_kind: "quote",
            payload_example: json!({ "quote": quote_example }),
        },
        EventDescriptor {
            name: "quote.updated",
            version: 1,
            label: "Devis modifié",
            subject_kind: "quote",
            payload_example: json!({
                "quote": quote_example,
                "changed_fields": ["title"],
                "previous": { "title": "Ancien titre" },
            }),
        },
        EventDescriptor {
            name: "quote.deleted",
            version: 1,
            label: "Devis supprimé",
            subject_kind: "quote",
            payload_example: json!({ "quote_id": "018f3b2a-0000-7000-8000-000000000001" }),
        },
    ];

    // Driven by the same exhaustive match the emission uses, so a new status
    // cannot gain an event without gaining its description in the same change.
    for status in QuoteStatus::ALL {
        let Some(name) = QuoteTransitioned::event_name(status) else {
            continue;
        };

        descriptors.push(EventDescriptor {
            name,
            version: 1,
            label: transition_label(status),
            subject_kind: "quote",
            payload_example: json!({
                "quote": quote_example,
                "from": "DRAFT",
                "to": status.as_str(),
            }),
        });
    }

    descriptors
}

fn transition_label(status: QuoteStatus) -> &'static str {
    match status {
        QuoteStatus::Sent => "Devis envoyé",
        QuoteStatus::Accepted => "Devis accepté",
        QuoteStatus::Declined => "Devis refusé",
        QuoteStatus::Cancelled => "Devis annulé",
        QuoteStatus::Draft => "Devis repassé en brouillon",
    }
}
