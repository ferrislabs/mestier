//! What the invoice aggregate publishes, mirroring `domain::quote::events`:
//! content, status transition, and (once #317/#318/#320 land) business acts
//! never overlap.

use common::UserId;
use events::{DomainEvent, EventDescriptor, EventSubject};
use serde_json::{Value, json};

use crate::{Invoice, InvoiceId, InvoicePayment, InvoicePaymentId, InvoiceStatus};

pub struct InvoiceCreated {
    pub invoice: Invoice,
}

impl DomainEvent for InvoiceCreated {
    fn name(&self) -> &'static str {
        "invoice.created"
    }

    fn version(&self) -> u16 {
        1
    }

    fn subject(&self) -> EventSubject {
        EventSubject::new("invoice", self.invoice.id.0)
    }

    fn payload(&self) -> Value {
        json!({ "invoice": invoice_payload(&self.invoice) })
    }
}

pub struct InvoiceUpdated {
    pub invoice: Invoice,
    pub changed_fields: Vec<&'static str>,
    pub previous: Value,
}

impl DomainEvent for InvoiceUpdated {
    fn name(&self) -> &'static str {
        "invoice.updated"
    }

    fn version(&self) -> u16 {
        1
    }

    fn subject(&self) -> EventSubject {
        EventSubject::new("invoice", self.invoice.id.0)
    }

    fn payload(&self) -> Value {
        json!({
            "invoice": invoice_payload(&self.invoice),
            "changed_fields": self.changed_fields,
            "previous": self.previous,
        })
    }
}

pub struct InvoiceDeleted {
    pub invoice_id: InvoiceId,
}

impl DomainEvent for InvoiceDeleted {
    fn name(&self) -> &'static str {
        "invoice.deleted"
    }

    fn version(&self) -> u16 {
        1
    }

    fn subject(&self) -> EventSubject {
        EventSubject::new("invoice", self.invoice_id.0)
    }

    fn payload(&self) -> Value {
        json!({ "invoice_id": self.invoice_id.0 })
    }
}

/// One transition of [`InvoiceStatus`], named after where it lands.
pub struct InvoiceTransitioned {
    pub invoice: Invoice,
    pub from: InvoiceStatus,
}

impl InvoiceTransitioned {
    /// `None` for a transition the product has no name for — today, `Draft`
    /// itself, which is never a transition's destination. Exhaustive over
    /// [`InvoiceStatus::ALL`] so a new status cannot be added without this
    /// match naming, or explicitly silencing, its event.
    pub fn event_name(to: InvoiceStatus) -> Option<&'static str> {
        match to {
            InvoiceStatus::Draft => None,
            InvoiceStatus::Issued => Some("invoice.issued"),
            InvoiceStatus::Paid => Some("invoice.paid"),
            InvoiceStatus::PartiallyPaid => Some("invoice.partially_paid"),
            InvoiceStatus::Cancelled => Some("invoice.cancelled"),
        }
    }
}

impl DomainEvent for InvoiceTransitioned {
    fn name(&self) -> &'static str {
        Self::event_name(self.invoice.status).expect("a nameless transition is never emitted")
    }

    fn version(&self) -> u16 {
        1
    }

    fn subject(&self) -> EventSubject {
        EventSubject::new("invoice", self.invoice.id.0)
    }

    fn payload(&self) -> Value {
        json!({
            "invoice": invoice_payload(&self.invoice),
            "from": self.from.as_str(),
            "to": self.invoice.status.as_str(),
        })
    }
}

/// A payment recorded against an invoice (#320).
pub struct InvoicePaymentRecorded {
    pub payment: InvoicePayment,
}

impl DomainEvent for InvoicePaymentRecorded {
    fn name(&self) -> &'static str {
        "invoice_payment.recorded"
    }

    fn version(&self) -> u16 {
        1
    }

    fn subject(&self) -> EventSubject {
        EventSubject::new("invoice_payment", self.payment.id.0)
    }

    fn payload(&self) -> Value {
        json!({ "payment": payment_payload(&self.payment) })
    }
}

/// A recorded payment soft-deleted (#320) — the row survives with who
/// deleted it and when, this event just announces the act.
pub struct InvoicePaymentDeleted {
    pub payment_id: InvoicePaymentId,
    pub invoice_id: InvoiceId,
    pub deleted_by: UserId,
}

impl DomainEvent for InvoicePaymentDeleted {
    fn name(&self) -> &'static str {
        "invoice_payment.deleted"
    }

    fn version(&self) -> u16 {
        1
    }

    fn subject(&self) -> EventSubject {
        EventSubject::new("invoice_payment", self.payment_id.0)
    }

    fn payload(&self) -> Value {
        json!({
            "payment_id": self.payment_id.0,
            "invoice_id": self.invoice_id.0,
            "deleted_by": self.deleted_by.0,
        })
    }
}

/// The serialized domain model, never the database row: renaming a column
/// must not reach a subscriber's automation.
fn invoice_payload(invoice: &Invoice) -> Value {
    json!({
        "id": invoice.id.0,
        "organization_id": invoice.organization_id.0,
        "number": invoice.number,
        "kind": invoice.kind.as_str(),
        "project_id": invoice.project_id.map(|id| id.0),
        "customer_id": invoice.customer_id.0,
        "customer_context_id": invoice.customer_context_id.0,
        "status": invoice.status.as_str(),
        "issued_at": invoice.issued_at,
        "due_at": invoice.due_at,
        "net_cents": invoice.net_cents,
        "vat_breakdown": invoice.vat_breakdown.iter().map(|line| json!({
            "rate_bp": line.rate_bp,
            "vat_cents": line.vat_cents,
        })).collect::<Vec<_>>(),
        "gross_cents": invoice.gross_cents,
        "created_at": invoice.created_at,
        "updated_at": invoice.updated_at,
    })
}

/// The serialized domain model, never the database row — same rule as
/// `invoice_payload`.
fn payment_payload(payment: &InvoicePayment) -> Value {
    json!({
        "id": payment.id.0,
        "organization_id": payment.organization_id.0,
        "invoice_id": payment.invoice_id.0,
        "amount_cents": payment.amount_cents,
        "paid_on": payment.paid_on,
        "method": payment.method,
        "reference": payment.reference,
        "note": payment.note,
        "recorded_by": payment.recorded_by.0,
        "created_at": payment.created_at,
        "updated_at": payment.updated_at,
    })
}

/// Every event this module can construct, as `(name, version)`. Test-only:
/// compared against the catalogue by the drift check in
/// `domain::automation::catalogue`.
#[cfg(test)]
pub fn emitted_events() -> Vec<(&'static str, u16)> {
    let mut emitted = vec![
        ("invoice.created", 1),
        ("invoice.updated", 1),
        ("invoice.deleted", 1),
        ("invoice_payment.recorded", 1),
        ("invoice_payment.deleted", 1),
    ];

    emitted.extend(
        InvoiceStatus::ALL
            .into_iter()
            .filter_map(InvoiceTransitioned::event_name)
            .map(|name| (name, 1)),
    );

    emitted
}

pub fn descriptors() -> Vec<EventDescriptor> {
    let invoice_example = json!({
        "id": "018f3b2a-0000-7000-8000-000000000001",
        "organization_id": "018f3b2a-0000-7000-8000-000000000002",
        "number": "FAC-2026-0001",
        "kind": "STANDARD",
        "project_id": "018f3b2a-0000-7000-8000-000000000005",
        "customer_id": "018f3b2a-0000-7000-8000-000000000003",
        "customer_context_id": "018f3b2a-0000-7000-8000-000000000004",
        "status": "DRAFT",
        "issued_at": null,
        "due_at": "2026-09-08T00:00:00Z",
        "net_cents": 550_000,
        "vat_breakdown": [],
        "gross_cents": 550_000,
        "created_at": "2026-08-23T09:00:00Z",
        "updated_at": "2026-08-23T09:00:00Z",
    });

    let mut descriptors = vec![
        EventDescriptor {
            name: "invoice.created",
            version: 1,
            label: "Facture créée",
            subject_kind: "invoice",
            payload_example: json!({ "invoice": invoice_example }),
        },
        EventDescriptor {
            name: "invoice.updated",
            version: 1,
            label: "Facture modifiée",
            subject_kind: "invoice",
            payload_example: json!({
                "invoice": invoice_example,
                "changed_fields": ["notes"],
                "previous": { "notes": null },
            }),
        },
        EventDescriptor {
            name: "invoice.deleted",
            version: 1,
            label: "Facture supprimée",
            subject_kind: "invoice",
            payload_example: json!({ "invoice_id": "018f3b2a-0000-7000-8000-000000000001" }),
        },
    ];

    for status in InvoiceStatus::ALL {
        let Some(name) = InvoiceTransitioned::event_name(status) else {
            continue;
        };

        descriptors.push(EventDescriptor {
            name,
            version: 1,
            label: transition_label(status),
            subject_kind: "invoice",
            payload_example: json!({
                "invoice": invoice_example,
                "from": "DRAFT",
                "to": status.as_str(),
            }),
        });
    }

    let payment_example = json!({
        "id": "018f3b2a-0000-7000-8000-000000000006",
        "organization_id": "018f3b2a-0000-7000-8000-000000000002",
        "invoice_id": "018f3b2a-0000-7000-8000-000000000001",
        "amount_cents": 275_000,
        "paid_on": "2026-09-01",
        "method": "Virement",
        "reference": "VIR-2026-0912",
        "note": null,
        "recorded_by": "018f3b2a-0000-7000-8000-000000000007",
        "created_at": "2026-09-01T09:00:00Z",
        "updated_at": "2026-09-01T09:00:00Z",
    });

    descriptors.push(EventDescriptor {
        name: "invoice_payment.recorded",
        version: 1,
        label: "Paiement enregistré",
        subject_kind: "invoice_payment",
        payload_example: json!({ "payment": payment_example }),
    });
    descriptors.push(EventDescriptor {
        name: "invoice_payment.deleted",
        version: 1,
        label: "Paiement supprimé",
        subject_kind: "invoice_payment",
        payload_example: json!({
            "payment_id": "018f3b2a-0000-7000-8000-000000000006",
            "invoice_id": "018f3b2a-0000-7000-8000-000000000001",
            "deleted_by": "018f3b2a-0000-7000-8000-000000000007",
        }),
    });

    descriptors
}

fn transition_label(status: InvoiceStatus) -> &'static str {
    match status {
        InvoiceStatus::Issued => "Facture émise",
        InvoiceStatus::Paid => "Facture payée",
        InvoiceStatus::PartiallyPaid => "Facture partiellement payée",
        InvoiceStatus::Cancelled => "Facture annulée",
        InvoiceStatus::Draft => "Facture repassée en brouillon",
    }
}
