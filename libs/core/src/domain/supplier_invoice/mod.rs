//! A received supplier invoice: somebody else's document, not a mirror of
//! the ones this product issues. Money going out was, until now, a number
//! typed into `tasks.expenses_cents` with a free-text label — this is the
//! document behind it. See #334, #336.
//!
//! Immutability is enforced at the type level: [`SupplierInvoice`] exposes
//! no mutating methods at all. The only type a mutation can land on is
//! [`SupplierInvoiceReview`], and it only ever touches `status` and
//! `notes` — our metadata about the document, never the document's own
//! fields (its supplier identity, its lines, its totals). Same device as
//! `invoice::DraftInvoice`, one directory over, except there is no
//! "wrong status to review" the way there is a "wrong status to edit": any
//! status can carry a note, so [`SupplierInvoiceReview::new`] needs no
//! runtime check, only [`SupplierInvoiceService::confirm`]/`reject` refuse
//! a transition that has already happened.

use std::{fmt::Display, str::FromStr};

use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::{OrganizationId, ProjectId};

pub mod commands;
pub mod events;
pub mod ports;
pub mod service;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema)]
pub struct SupplierId(pub Uuid);

impl FromStr for SupplierId {
    type Err = uuid::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Uuid::from_str(s).map(SupplierId)
    }
}

impl Display for SupplierId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// A merchant an organization buys from repeatedly. Deliberately thin: this
/// issue only needs the row to exist for [`SupplierInvoice::supplier_id`] to
/// point at, nothing here manages recognising or creating suppliers yet —
/// left for whichever later issue actually needs that flow.
#[derive(Debug, Clone, PartialEq)]
pub struct Supplier {
    pub id: SupplierId,
    pub organization_id: OrganizationId,
    pub name: String,
    pub registration_number: Option<String>,
    pub vat_number: Option<String>,
    pub deleted_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema)]
pub struct SupplierInvoiceId(pub Uuid);

impl FromStr for SupplierInvoiceId {
    type Err = uuid::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Uuid::from_str(s).map(SupplierInvoiceId)
    }
}

impl Display for SupplierInvoiceId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// A parsed (or manually entered) document is a proposal until a human
/// confirms it — nothing here is a cost until somebody said so. `Received`
/// is the only status a new invoice can start in; `Confirmed`/`Rejected`
/// are terminal, set exactly once by [`SupplierInvoiceService::confirm`]/
/// `reject`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SupplierInvoiceStatus {
    Received,
    Confirmed,
    Rejected,
}

impl SupplierInvoiceStatus {
    /// Every variant, for exhaustive iteration — same device as
    /// `InvoiceStatus::ALL`, for the same reason: `events::event_name`
    /// matches exhaustively, so a new status cannot be added without this
    /// match naming, or explicitly silencing, its event.
    pub const ALL: [SupplierInvoiceStatus; 3] = [
        SupplierInvoiceStatus::Received,
        SupplierInvoiceStatus::Confirmed,
        SupplierInvoiceStatus::Rejected,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Received => "RECEIVED",
            Self::Confirmed => "CONFIRMED",
            Self::Rejected => "REJECTED",
        }
    }
}

impl Display for SupplierInvoiceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl FromStr for SupplierInvoiceStatus {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "RECEIVED" => Ok(Self::Received),
            "CONFIRMED" => Ok(Self::Confirmed),
            "REJECTED" => Ok(Self::Rejected),
            other => Err(format!("invalid supplier invoice status `{other}`")),
        }
    }
}

/// How a [`SupplierInvoice`] row came to exist. A plain string, not an
/// exhaustive enum: unlike `SupplierInvoiceStatus` this is expected to grow
/// (a PDP transport is a real future value, see #334's "needs verification"
/// note), and closing over the known values today would make every future
/// source a migration on the Rust side too, not just the database's.
/// `as_str`/`FromStr` still exist so callers get the same ergonomics as a
/// real enum for the sources that do exist.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SupplierInvoiceSource {
    /// Typed in by hand, no document parser involved.
    Manual,
    /// Parsed from a Factur-X file — #337's own scope, not reachable
    /// through this issue's service.
    FacturX,
}

impl SupplierInvoiceSource {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Manual => "MANUAL",
            Self::FacturX => "FACTUR_X",
        }
    }
}

impl Display for SupplierInvoiceSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl FromStr for SupplierInvoiceSource {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "MANUAL" => Ok(Self::Manual),
            "FACTUR_X" => Ok(Self::FacturX),
            other => Err(format!("invalid supplier invoice source `{other}`")),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema)]
pub struct SupplierInvoiceLineId(pub Uuid);

impl FromStr for SupplierInvoiceLineId {
    type Err = uuid::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Uuid::from_str(s).map(SupplierInvoiceLineId)
    }
}

impl Display for SupplierInvoiceLineId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SupplierInvoiceLine {
    pub id: SupplierInvoiceLineId,
    pub organization_id: OrganizationId,
    pub supplier_invoice_id: SupplierInvoiceId,
    pub label: String,
    pub quantity: Decimal,
    pub unit: Option<String>,
    pub unit_price_cents: i32,
    /// Stored exactly as printed on the document, never derived from
    /// `quantity * unit_price_cents`: their own rounding is part of what
    /// was received, not a value we get to silently correct.
    pub line_total_cents: i32,
    pub vat_rate_basis_points: Option<i32>,
    pub position: i32,
    pub deleted_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// One rate's contribution to a received invoice's VAT, mirroring
/// `InvoiceVatBreakdownLine` — our own reading of what the supplier's rates
/// add up to, computed from [`SupplierInvoiceLine::line_total_cents`], not
/// a figure copied from the document.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SupplierInvoiceVatBreakdownLine {
    pub rate_bp: i32,
    pub vat_cents: i32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SupplierInvoice {
    pub id: SupplierInvoiceId,
    pub organization_id: OrganizationId,
    /// `None` until the merchant is recognised — left for whichever later
    /// issue actually manages [`Supplier`] rows. The identity fields below
    /// are what the document itself says, independent of this link.
    pub supplier_id: Option<SupplierId>,
    pub supplier_name: String,
    pub supplier_registration_number: Option<String>,
    pub supplier_vat_number: Option<String>,
    /// As issued by them, never allocated by us — contrast `Invoice::number`.
    pub number: String,
    pub issued_on: NaiveDate,
    pub due_on: Option<NaiveDate>,
    pub received_at: DateTime<Utc>,
    pub source: SupplierInvoiceSource,
    pub status: SupplierInvoiceStatus,
    pub currency: String,
    /// The original file this document was parsed from (#339) — "the file
    /// is stored, not only parsed, the original is the legal record and
    /// the parse is a derivation of it." `None` for a manually entered
    /// invoice, which has no file behind it at all. Set once, at creation
    /// — there is no setter on [`SupplierInvoiceReview`] for it, the same
    /// way the document's other fields are immutable once persisted.
    pub source_file_key: Option<String>,
    /// Always `Some` exactly when `source_file_key` is — enforced at the
    /// database (`chk_supplier_invoices_source_file_pair`), not just here.
    pub source_file_mime_type: Option<String>,
    /// Our metadata about the document, not part of it — the one other
    /// field [`SupplierInvoiceReview`] is allowed to touch, alongside
    /// `status`.
    pub notes: Option<String>,
    pub net_cents: i32,
    pub vat_breakdown: Vec<SupplierInvoiceVatBreakdownLine>,
    pub gross_cents: i32,
    pub lines: Vec<SupplierInvoiceLine>,
    pub deleted_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema)]
pub struct SupplierInvoiceLineAllocationId(pub Uuid);

impl FromStr for SupplierInvoiceLineAllocationId {
    type Err = uuid::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Uuid::from_str(s).map(SupplierInvoiceLineAllocationId)
    }
}

impl Display for SupplierInvoiceLineAllocationId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// One project's share of a supplier invoice line's own cost — see #338.
///
/// `amount_cents` is net of VAT, a slice of the line's own
/// `line_total_cents`, never the gross figure: whether the real cost is
/// this net amount or a grossed-up one depends on the organization's own
/// VAT status, and that decision is made once, at report time, by
/// `profitability::service` — not carried per allocation.
///
/// A line may be split across several projects, or left partly (or wholly)
/// unallocated — general overhead the business absorbs, not a project's own
/// cost. The one invariant, that a line's allocations never exceed what it
/// is worth, is enforced by a database trigger
/// (`supplier_invoice_line_allocations_enforce_line_total`) rather than at
/// this type's construction: a per-row check has no way to see the sum
/// across a line's siblings, only an aggregate query does, and the
/// database already runs one on every write.
#[derive(Debug, Clone, PartialEq)]
pub struct SupplierInvoiceLineAllocation {
    pub id: SupplierInvoiceLineAllocationId,
    pub organization_id: OrganizationId,
    pub supplier_invoice_line_id: SupplierInvoiceLineId,
    pub project_id: ProjectId,
    pub amount_cents: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// The only type a mutation can land on. Unlike `invoice::DraftInvoice`,
/// construction carries no runtime check — any status can take a note, so
/// there is no "wrong state to review" the way there is a "wrong state to
/// edit" — the guard that matters (refusing to confirm/reject a document
/// twice) lives in [`SupplierInvoiceService`], which decides *whether* to
/// reach for one of these, not in the type itself.
#[derive(Debug, Clone, PartialEq)]
pub struct SupplierInvoiceReview(SupplierInvoice);

impl SupplierInvoiceReview {
    pub fn new(invoice: SupplierInvoice) -> Self {
        Self(invoice)
    }

    pub fn invoice(&self) -> &SupplierInvoice {
        &self.0
    }

    pub fn into_invoice(self) -> SupplierInvoice {
        self.0
    }

    pub fn set_status(&mut self, status: SupplierInvoiceStatus) {
        self.0.status = status;
    }

    pub fn set_notes(&mut self, notes: Option<String>) {
        self.0.notes = notes;
    }

    pub fn touch(&mut self, now: DateTime<Utc>) {
        self.0.updated_at = now;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn supplier_invoice_status_round_trips_through_its_string_form() {
        for status in SupplierInvoiceStatus::ALL {
            assert_eq!(
                status.as_str().parse::<SupplierInvoiceStatus>().unwrap(),
                status
            );
        }
    }

    #[test]
    fn supplier_invoice_status_rejects_unknown_values() {
        assert!("APPROVED".parse::<SupplierInvoiceStatus>().is_err());
    }

    #[test]
    fn supplier_invoice_source_round_trips_through_its_string_form() {
        for source in [
            SupplierInvoiceSource::Manual,
            SupplierInvoiceSource::FacturX,
        ] {
            assert_eq!(
                source.as_str().parse::<SupplierInvoiceSource>().unwrap(),
                source
            );
        }
    }

    #[test]
    fn supplier_invoice_id_parses_uuid() {
        let uuid = Uuid::new_v4();
        let parsed = SupplierInvoiceId::from_str(&uuid.to_string()).unwrap();

        assert_eq!(parsed.0, uuid);
    }

    #[test]
    fn supplier_invoice_line_allocation_id_parses_uuid() {
        let uuid = Uuid::new_v4();
        let parsed = SupplierInvoiceLineAllocationId::from_str(&uuid.to_string()).unwrap();

        assert_eq!(parsed.0, uuid);
    }
}
