//! An invoice: a document frozen the moment it is issued.
//!
//! Not a quote in another status — a quote is edited until accepted, an
//! invoice is immutable from the instant it leaves `Draft`. Modelling both
//! as one table with a mutability rule keyed on status is exactly the kind
//! of rule that gets forgotten in one code path (today the PDF export,
//! tomorrow a bulk edit), so an invoice is its own aggregate. See #316.
//!
//! Immutability is enforced at the type level, not by a runtime guard
//! sprinkled through every mutating call site: [`Invoice`] exposes no
//! mutating methods at all, and [`DraftInvoice`] — the only type any
//! mutation lands on — can only be obtained from an invoice whose status is
//! `Draft`. `InvoiceRepository::update_draft` accepts a `&DraftInvoice`, not
//! a `&Invoice`, so a call site holding an issued invoice cannot even
//! compile the attempt; it has to go through [`DraftInvoice::try_from_invoice`]
//! first, which is where the one runtime check lives.

use std::{fmt::Display, str::FromStr};

use chrono::{DateTime, Utc};
use common::CoreError;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::{
    CustomerContextId, CustomerId, LegalIdentity, OrganizationAddress, OrganizationId, ProjectId,
};

pub mod commands;
pub mod events;
pub mod ports;
pub mod service;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema)]
pub struct InvoiceId(pub Uuid);

impl FromStr for InvoiceId {
    type Err = uuid::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Uuid::from_str(s).map(InvoiceId)
    }
}

impl Display for InvoiceId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema)]
pub struct InvoiceLineId(pub Uuid);

impl FromStr for InvoiceLineId {
    type Err = uuid::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Uuid::from_str(s).map(InvoiceLineId)
    }
}

impl Display for InvoiceLineId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Payment status (`Paid`/`PartiallyPaid`) is never set by hand: two sources
/// for "is this paid" is one too many. It is derived from recorded payments
/// and credit notes — wired up in #320 — and every other status is set by an
/// explicit act (create a draft, issue it, cancel it).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum InvoiceStatus {
    Draft,
    Issued,
    Paid,
    PartiallyPaid,
    Cancelled,
}

impl InvoiceStatus {
    /// Every variant, for exhaustive iteration. Adding one here is not
    /// enforced by the compiler, but naming its event is: `event_name`
    /// (see `events::InvoiceTransitioned`) matches exhaustively, so a new
    /// status cannot be ignored silently.
    pub const ALL: [InvoiceStatus; 5] = [
        InvoiceStatus::Draft,
        InvoiceStatus::Issued,
        InvoiceStatus::Paid,
        InvoiceStatus::PartiallyPaid,
        InvoiceStatus::Cancelled,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Draft => "DRAFT",
            Self::Issued => "ISSUED",
            Self::Paid => "PAID",
            Self::PartiallyPaid => "PARTIALLY_PAID",
            Self::Cancelled => "CANCELLED",
        }
    }
}

impl Display for InvoiceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl FromStr for InvoiceStatus {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "DRAFT" => Ok(Self::Draft),
            "ISSUED" => Ok(Self::Issued),
            "PAID" => Ok(Self::Paid),
            "PARTIALLY_PAID" => Ok(Self::PartiallyPaid),
            "CANCELLED" => Ok(Self::Cancelled),
            other => Err(format!("invalid invoice status `{other}`")),
        }
    }
}

/// How an invoice's lines came to be. One document, several ways of
/// computing its lines — the ruling from #317's discussion applies to the
/// type from the start, even though only `Standard` is reachable through
/// this issue's own service: #317 (deposit / final, deducting what was
/// already invoiced) and #318 (credit notes) both need a variant and
/// neither carries a migration of its own, so the whole shape is modelled
/// here rather than ALTERed in piecemeal. Nothing about this type assumes a
/// project has at most one line-generating strategy, which is the
/// constraint progress billing (#333, unscheduled) needs preserved.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum InvoiceKind {
    Standard,
    Deposit,
    Final,
    CreditNote,
}

impl InvoiceKind {
    pub const ALL: [InvoiceKind; 4] = [
        InvoiceKind::Standard,
        InvoiceKind::Deposit,
        InvoiceKind::Final,
        InvoiceKind::CreditNote,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Standard => "STANDARD",
            Self::Deposit => "DEPOSIT",
            Self::Final => "FINAL",
            Self::CreditNote => "CREDIT_NOTE",
        }
    }
}

impl Display for InvoiceKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl FromStr for InvoiceKind {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "STANDARD" => Ok(Self::Standard),
            "DEPOSIT" => Ok(Self::Deposit),
            "FINAL" => Ok(Self::Final),
            "CREDIT_NOTE" => Ok(Self::CreditNote),
            other => Err(format!("invalid invoice kind `{other}`")),
        }
    }
}

/// The nature of the operation an invoice bills for — one of the mentions
/// the e-invoicing reform effective 1 September 2026 requires (#341).
/// `None` on the invoice until the caller states it; nothing here defaults
/// to a guess.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum OperationNature {
    Goods,
    Services,
    Both,
}

impl OperationNature {
    pub const ALL: [OperationNature; 3] = [
        OperationNature::Goods,
        OperationNature::Services,
        OperationNature::Both,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Goods => "GOODS",
            Self::Services => "SERVICES",
            Self::Both => "BOTH",
        }
    }
}

impl Display for OperationNature {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl FromStr for OperationNature {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "GOODS" => Ok(Self::Goods),
            "SERVICES" => Ok(Self::Services),
            "BOTH" => Ok(Self::Both),
            other => Err(format!("invalid invoice operation nature `{other}`")),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct InvoiceLine {
    pub id: InvoiceLineId,
    pub organization_id: OrganizationId,
    pub invoice_id: InvoiceId,
    /// Copied text, never a foreign key to `quote_lines`: editing a quote
    /// after invoicing must not rewrite an issued invoice.
    pub label: String,
    pub quantity: Decimal,
    pub unit_price_cents: i32,
    /// Basis points (2000 = 20 %, 550 = 5.5 %). Travels with the line for the
    /// same reason it does on a quote line: a document already issued must
    /// not change because a rate was edited afterwards.
    pub vat_rate_basis_points: Option<i32>,
    /// Display order. Lines are a snapshot, not a set — reordering after
    /// creation is a new draft edit, not a live reorder endpoint.
    pub position: i32,
    pub deleted_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// One rate's contribution to an invoice's VAT, mirroring
/// `QuoteVatBreakdownLine`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InvoiceVatBreakdownLine {
    pub rate_bp: i32,
    pub vat_cents: i32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Invoice {
    pub id: InvoiceId,
    pub organization_id: OrganizationId,
    /// Allocated when the invoice first leaves `Draft`, gapless and unique
    /// per organization, never reallocated afterwards. `None` on a draft.
    pub number: Option<String>,
    pub kind: InvoiceKind,
    /// A project with no quote can still be invoiced (#317): the quote is
    /// the margin denominator, not a precondition for billing. This can also
    /// be `None` for an invoice raised outside a project's billing flow.
    pub project_id: Option<ProjectId>,
    pub customer_id: CustomerId,
    pub customer_context_id: CustomerContextId,
    pub status: InvoiceStatus,
    /// Set exactly when the invoice leaves `Draft`, never after.
    pub issued_at: Option<DateTime<Utc>>,
    pub due_at: Option<DateTime<Utc>>,
    pub notes: Option<String>,
    /// The nature of the operation this invoice bills for — see
    /// `OperationNature`. `None` until the caller states it; #341 adds this
    /// nullable so existing invoices, and drafts still being edited, are
    /// not forced to guess.
    pub operation_nature: Option<OperationNature>,
    /// Where the goods or services were delivered, only when it differs
    /// from the customer's own address — the fourth e-invoicing mention
    /// #341 adds. `None` means "same as the customer", not "unknown".
    pub delivery_address: Option<OrganizationAddress>,
    pub net_cents: i32,
    pub vat_breakdown: Vec<InvoiceVatBreakdownLine>,
    pub gross_cents: i32,
    /// The issuer's `LegalIdentity` as it was the instant this invoice was
    /// issued — never looked up live from the organization afterwards, so a
    /// later change to the organization's legal details cannot rewrite a
    /// document already sent. `None` on a draft.
    pub issuer_identity: Option<LegalIdentity>,
    pub lines: Vec<InvoiceLine>,
    /// The invoice this one corrects, set exactly when `kind` is
    /// `CreditNote` and `None` for every other kind — see #318. Set once at
    /// construction, never through `DraftInvoice`: a credit note's target is
    /// fixed the instant it is built, not something an edit-in-place flow
    /// ever needs to change.
    pub source_invoice_id: Option<InvoiceId>,
    pub deleted_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// The only type a mutation can land on. Obtained exclusively through
/// [`DraftInvoice::try_from_invoice`] (existing invoice, checked once) or
/// [`DraftInvoice::new`] (brand new, always a draft by construction) — there
/// is no path from an issued `Invoice` to a `DraftInvoice` that skips the
/// check, and no mutating method exists on `Invoice` itself.
#[derive(Debug, Clone, PartialEq)]
pub struct DraftInvoice(Invoice);

impl DraftInvoice {
    /// `Err` when `invoice.status != Draft`. This is the one runtime check
    /// standing between a persisted invoice and the ability to edit it —
    /// everything downstream of it (`InvoiceRepository::update_draft`, the
    /// domain's own setters) is a compile-time guarantee once satisfied.
    pub fn try_from_invoice(invoice: Invoice) -> Result<Self, CoreError> {
        if invoice.status != InvoiceStatus::Draft {
            return Err(CoreError::Conflict(format!(
                "invoice {} is {} and cannot be edited; only a draft can be",
                invoice.id, invoice.status
            )));
        }

        Ok(Self(invoice))
    }

    pub fn invoice(&self) -> &Invoice {
        &self.0
    }

    pub fn into_invoice(self) -> Invoice {
        self.0
    }

    pub fn set_kind(&mut self, kind: InvoiceKind) {
        self.0.kind = kind;
    }

    pub fn set_project(&mut self, project_id: Option<ProjectId>) {
        self.0.project_id = project_id;
    }

    pub fn set_customer(
        &mut self,
        customer_id: CustomerId,
        customer_context_id: CustomerContextId,
    ) {
        self.0.customer_id = customer_id;
        self.0.customer_context_id = customer_context_id;
    }

    pub fn set_due_at(&mut self, due_at: Option<DateTime<Utc>>) {
        self.0.due_at = due_at;
    }

    pub fn set_notes(&mut self, notes: Option<String>) {
        self.0.notes = notes;
    }

    pub fn set_operation_nature(&mut self, operation_nature: Option<OperationNature>) {
        self.0.operation_nature = operation_nature;
    }

    pub fn set_delivery_address(&mut self, delivery_address: Option<OrganizationAddress>) {
        self.0.delivery_address = delivery_address;
    }

    /// Replaces the lines and their computed totals in one step: the two
    /// must never be set independently, or a line list and its own total
    /// could disagree.
    pub fn set_lines_and_totals(
        &mut self,
        lines: Vec<InvoiceLine>,
        net_cents: i32,
        vat_breakdown: Vec<InvoiceVatBreakdownLine>,
        gross_cents: i32,
    ) {
        self.0.lines = lines;
        self.0.net_cents = net_cents;
        self.0.vat_breakdown = vat_breakdown;
        self.0.gross_cents = gross_cents;
    }

    pub fn touch(&mut self, now: DateTime<Utc>) {
        self.0.updated_at = now;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invoice_status_parses_known_values() {
        assert_eq!(
            "DRAFT".parse::<InvoiceStatus>().unwrap(),
            InvoiceStatus::Draft
        );
        assert_eq!(
            "PARTIALLY_PAID".parse::<InvoiceStatus>().unwrap(),
            InvoiceStatus::PartiallyPaid
        );
    }

    #[test]
    fn invoice_status_rejects_unknown_values() {
        assert!("APPROVED".parse::<InvoiceStatus>().is_err());
    }

    #[test]
    fn invoice_kind_round_trips_through_its_string_form() {
        for kind in InvoiceKind::ALL {
            assert_eq!(kind.as_str().parse::<InvoiceKind>().unwrap(), kind);
        }
    }

    #[test]
    fn invoice_id_parses_uuid() {
        let uuid = Uuid::new_v4();
        let parsed = InvoiceId::from_str(&uuid.to_string()).unwrap();

        assert_eq!(parsed.0, uuid);
    }
}
