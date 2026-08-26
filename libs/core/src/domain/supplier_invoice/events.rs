//! What the supplier invoice aggregate publishes, mirroring
//! `domain::invoice::events`: content and status transition never overlap.

use events::{DomainEvent, EventDescriptor, EventSubject};
use serde_json::{Value, json};

use crate::{SupplierInvoice, SupplierInvoiceStatus};

pub struct SupplierInvoiceReceived {
    pub invoice: SupplierInvoice,
}

impl DomainEvent for SupplierInvoiceReceived {
    fn name(&self) -> &'static str {
        "supplier_invoice.received"
    }

    fn version(&self) -> u16 {
        1
    }

    fn subject(&self) -> EventSubject {
        EventSubject::new("supplier_invoice", self.invoice.id.0)
    }

    fn payload(&self) -> Value {
        json!({ "supplier_invoice": supplier_invoice_payload(&self.invoice) })
    }
}

/// One transition of [`SupplierInvoiceStatus`], named after where it lands
/// — mirrors `InvoiceTransitioned`.
pub struct SupplierInvoiceTransitioned {
    pub invoice: SupplierInvoice,
    pub from: SupplierInvoiceStatus,
}

impl SupplierInvoiceTransitioned {
    /// `None` for a transition the product has no name for — today,
    /// `Received` itself, which is never a transition's destination.
    /// Exhaustive over [`SupplierInvoiceStatus::ALL`] so a new status
    /// cannot be added without this match naming, or explicitly silencing,
    /// its event.
    pub fn event_name(to: SupplierInvoiceStatus) -> Option<&'static str> {
        match to {
            SupplierInvoiceStatus::Received => None,
            SupplierInvoiceStatus::Confirmed => Some("supplier_invoice.confirmed"),
            SupplierInvoiceStatus::Rejected => Some("supplier_invoice.rejected"),
        }
    }
}

impl DomainEvent for SupplierInvoiceTransitioned {
    fn name(&self) -> &'static str {
        Self::event_name(self.invoice.status).expect("a nameless transition is never emitted")
    }

    fn version(&self) -> u16 {
        1
    }

    fn subject(&self) -> EventSubject {
        EventSubject::new("supplier_invoice", self.invoice.id.0)
    }

    fn payload(&self) -> Value {
        json!({
            "supplier_invoice": supplier_invoice_payload(&self.invoice),
            "from": self.from.as_str(),
            "to": self.invoice.status.as_str(),
        })
    }
}

/// The serialized domain model, never the database row: renaming a column
/// must not reach a subscriber's automation.
fn supplier_invoice_payload(invoice: &SupplierInvoice) -> Value {
    json!({
        "id": invoice.id.0,
        "organization_id": invoice.organization_id.0,
        "supplier_id": invoice.supplier_id.map(|id| id.0),
        "supplier_name": invoice.supplier_name,
        "number": invoice.number,
        "issued_on": invoice.issued_on,
        "due_on": invoice.due_on,
        "received_at": invoice.received_at,
        "source": invoice.source.as_str(),
        "status": invoice.status.as_str(),
        "currency": invoice.currency,
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

/// Every event this module can construct, as `(name, version)`. Test-only:
/// compared against the catalogue by the drift check in
/// `domain::automation::catalogue`.
#[cfg(test)]
pub fn emitted_events() -> Vec<(&'static str, u16)> {
    let mut emitted = vec![("supplier_invoice.received", 1)];

    emitted.extend(
        SupplierInvoiceStatus::ALL
            .into_iter()
            .filter_map(SupplierInvoiceTransitioned::event_name)
            .map(|name| (name, 1)),
    );

    emitted
}

pub fn descriptors() -> Vec<EventDescriptor> {
    let supplier_invoice_example = json!({
        "id": "018f3b2a-0000-7000-8000-000000000010",
        "organization_id": "018f3b2a-0000-7000-8000-000000000002",
        "supplier_id": null,
        "supplier_name": "Point P",
        "number": "F-2026-4471",
        "issued_on": "2026-08-20",
        "due_on": "2026-09-19",
        "received_at": "2026-08-25T09:00:00Z",
        "source": "MANUAL",
        "status": "RECEIVED",
        "currency": "EUR",
        "net_cents": 45_000,
        "vat_breakdown": [{ "rate_bp": 2000, "vat_cents": 9_000 }],
        "gross_cents": 54_000,
        "created_at": "2026-08-25T09:00:00Z",
        "updated_at": "2026-08-25T09:00:00Z",
    });

    let mut descriptors = vec![EventDescriptor {
        name: "supplier_invoice.received",
        version: 1,
        label: "Facture fournisseur reçue",
        subject_kind: "supplier_invoice",
        payload_example: json!({ "supplier_invoice": supplier_invoice_example }),
    }];

    for status in SupplierInvoiceStatus::ALL {
        let Some(name) = SupplierInvoiceTransitioned::event_name(status) else {
            continue;
        };

        descriptors.push(EventDescriptor {
            name,
            version: 1,
            label: transition_label(status),
            subject_kind: "supplier_invoice",
            payload_example: json!({
                "supplier_invoice": supplier_invoice_example,
                "from": "RECEIVED",
                "to": status.as_str(),
            }),
        });
    }

    descriptors
}

fn transition_label(status: SupplierInvoiceStatus) -> &'static str {
    match status {
        SupplierInvoiceStatus::Confirmed => "Facture fournisseur confirmée",
        SupplierInvoiceStatus::Rejected => "Facture fournisseur rejetée",
        SupplierInvoiceStatus::Received => "Facture fournisseur reçue",
    }
}
