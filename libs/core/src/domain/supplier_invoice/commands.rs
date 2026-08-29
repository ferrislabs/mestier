use chrono::NaiveDate;
use rust_decimal::Decimal;

use crate::{
    OrganizationId, ProjectId, SupplierInvoiceId, SupplierInvoiceLineId, SupplierInvoiceSource,
};

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
    /// The stored original file this record was parsed from (#339) — both
    /// `Some` or both `None`, enforced at the database
    /// (`chk_supplier_invoices_source_file_pair`); a manually created
    /// invoice passes `None` for both.
    pub source_file_key: Option<String>,
    pub source_file_mime_type: Option<String>,
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

/// Attributes part (or all) of a line's cost to a project — see #338.
///
/// `supplier_invoice_id` is carried alongside the line id rather than
/// looked up from it, the same shape `RecordInvoicePaymentCommand` and its
/// siblings already use: the caller already has the invoice loaded to pick
/// a line from, and the service uses it to fetch the line through
/// `SupplierInvoiceRepository`'s existing `find_by_id`, needing no new
/// query on the frozen supplier invoice port. Allowed on a document of any
/// status, `Received` included: entering an allocation is bookkeeping,
/// independent of whether the invoice itself has been reviewed yet — only
/// a `Confirmed` invoice's allocations ever move a profitability number
/// (see `profitability::service::build_report`).
#[derive(Debug, Clone)]
pub struct AllocateSupplierInvoiceLineCommand {
    pub organization_id: OrganizationId,
    pub supplier_invoice_id: SupplierInvoiceId,
    pub supplier_invoice_line_id: SupplierInvoiceLineId,
    pub project_id: ProjectId,
    /// Net of VAT, same sign as the target line's own `line_total_cents` —
    /// see the doc comment on `SupplierInvoiceLineAllocation::amount_cents`.
    pub amount_cents: i32,
}

/// Metadata-only edit (#339's `PATCH .../supplier-invoices/{id}`) — the
/// document's own fields have no path to this command at all, only
/// `notes` does. Same "our metadata, not the document" boundary
/// `SupplierInvoiceReview` already draws for `confirm`/`reject`; this is
/// the same edit with no status transition attached, for a reviewer who
/// wants to leave or clear a note without accepting or refusing anything.
#[derive(Debug, Clone)]
pub struct UpdateSupplierInvoiceNotesCommand {
    pub id: SupplierInvoiceId,
    pub notes: Option<String>,
}

/// One project's share of a line, as part of a full replacement (#339's
/// `PUT .../supplier-invoice-lines/{line_id}/allocations`) — mirrors
/// `Task::assignments`: the body is the complete list for that line, not a
/// delta, the same contract for the same reason.
#[derive(Debug, Clone)]
pub struct LineAllocationShare {
    pub project_id: ProjectId,
    pub amount_cents: i32,
}

#[derive(Debug, Clone)]
pub struct ReplaceSupplierInvoiceLineAllocationsCommand {
    pub organization_id: OrganizationId,
    /// Carried alongside the line id for the same reason
    /// `AllocateSupplierInvoiceLineCommand` carries it: the service
    /// resolves the line through `SupplierInvoiceRepository::find_by_id`,
    /// which loads a whole invoice, not a bare line.
    pub supplier_invoice_id: SupplierInvoiceId,
    pub supplier_invoice_line_id: SupplierInvoiceLineId,
    pub allocations: Vec<LineAllocationShare>,
}
