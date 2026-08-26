use chrono::NaiveDate;
use rust_decimal::Decimal;

use crate::{OrganizationId, SupplierInvoiceId, SupplierInvoiceSource};

#[derive(Debug, Clone)]
pub struct SupplierInvoiceLineCommand {
    pub label: String,
    pub quantity: Decimal,
    pub unit: Option<String>,
    pub unit_price_cents: i32,
    /// Stored exactly as given, never derived from `quantity * unit_price`
    /// — see the doc comment on `SupplierInvoiceLine::line_total_cents`.
    pub line_total_cents: i32,
    pub vat_rate_basis_points: Option<i32>,
}

/// Records a received document. `supplier_id` is not a field here: an
/// invoice must be storable before its merchant is recognised, and nothing
/// in this issue's own scope ever assigns one — see the doc comment on
/// `SupplierInvoice::supplier_id`.
#[derive(Debug, Clone)]
pub struct CreateSupplierInvoiceCommand {
    pub organization_id: OrganizationId,
    pub supplier_name: String,
    pub supplier_registration_number: Option<String>,
    pub supplier_vat_number: Option<String>,
    pub number: String,
    pub issued_on: NaiveDate,
    pub due_on: Option<NaiveDate>,
    pub source: SupplierInvoiceSource,
    pub currency: String,
    pub lines: Vec<SupplierInvoiceLineCommand>,
}

/// Accepts a `Received` document as-is. Refused by
/// `SupplierInvoiceService::confirm` on anything else — a document is
/// reviewed exactly once, the same rule `cancel_invoice` enforces one
/// directory over.
#[derive(Debug, Clone)]
pub struct ConfirmSupplierInvoiceCommand {
    pub id: SupplierInvoiceId,
    /// A reviewer's note left at the moment of confirming, if any. Not
    /// required: most confirmations need no comment.
    pub notes: Option<String>,
}

/// Refuses a `Received` document. Same one-shot rule as
/// [`ConfirmSupplierInvoiceCommand`] — a rejected document stays rejected.
#[derive(Debug, Clone)]
pub struct RejectSupplierInvoiceCommand {
    pub id: SupplierInvoiceId,
    /// Why, if the reviewer said so. Unlike confirming, a rejection with no
    /// explanation is usually a gap, but this issue does not force one.
    pub notes: Option<String>,
}
