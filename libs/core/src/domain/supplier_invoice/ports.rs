use chrono::NaiveDate;
use common::CoreError;
use rust_decimal::Decimal;

use crate::{
    OrganizationId, ProjectId, ProjectSupplierCostLine, SupplierInvoice, SupplierInvoiceId,
    SupplierInvoiceLine, SupplierInvoiceLineAllocation, SupplierInvoiceLineAllocationId,
    SupplierInvoiceLineId, SupplierInvoiceReview,
};

#[cfg_attr(any(test, feature = "mock"), mockall::automock)]
pub trait SupplierInvoiceRepository: Send {
    fn insert(
        &mut self,
        invoice: &SupplierInvoice,
    ) -> impl Future<Output = Result<SupplierInvoice, CoreError>> + Send;

    fn find_by_id(
        &mut self,
        id: SupplierInvoiceId,
    ) -> impl Future<Output = Result<Option<SupplierInvoice>, CoreError>> + Send;

    fn list_by_organization(
        &mut self,
        organization_id: OrganizationId,
        limit: u64,
        offset: u64,
    ) -> impl Future<Output = Result<(Vec<SupplierInvoice>, u64), CoreError>> + Send;

    /// Only ever called with a [`SupplierInvoiceReview`]: persists `status`
    /// and `notes` alone, never the document's own fields. The type is what
    /// keeps a call site holding the wrong thing from compiling; the
    /// implementation's `UPDATE` naming exactly those two columns is what
    /// keeps it true even if a future caller reaches for the struct
    /// directly.
    fn update_review(
        &mut self,
        review: &SupplierInvoiceReview,
    ) -> impl Future<Output = Result<SupplierInvoice, CoreError>> + Send;

    /// #337's most important rule: the same invoice imported twice must be
    /// refused, not silently create a second cost. `identifier` is whichever
    /// of registration number, VAT number, or supplier name distinguishes
    /// the merchant — see [`supplier_identifier`] for the preference order,
    /// which `PgSupplierInvoiceRepository`'s implementation applies to the
    /// stored row with the same `COALESCE` order so both sides of the
    /// comparison agree. `number` is the supplier's own invoice number, never
    /// one we allocate.
    fn exists_with_duplicate_key(
        &mut self,
        organization_id: OrganizationId,
        number: &str,
        identifier: &str,
    ) -> impl Future<Output = Result<bool, CoreError>> + Send;

    /// Resolves a line on its own, `None` if it never existed or its
    /// parent invoice was soft-deleted — #339's full-replace `PUT
    /// .../supplier-invoice-lines/{line_id}/allocations` has only the bare
    /// line id (CLAUDE.md: bare ids derive their organization from the
    /// loaded row) and needs the line's own `organization_id` and
    /// `supplier_invoice_id` before it can build a
    /// [`crate::ReplaceSupplierInvoiceLineAllocationsCommand`].
    fn find_line_by_id(
        &mut self,
        line_id: SupplierInvoiceLineId,
    ) -> impl Future<Output = Result<Option<SupplierInvoiceLine>, CoreError>> + Send;
}

/// A document read out of a supplier's file — Factur-X today, see
/// [`SupplierInvoiceParser`] — before it becomes a
/// [`crate::domain::supplier_invoice::commands::CreateSupplierInvoiceCommand`].
/// Deliberately not that command itself: the command has no field for what
/// the document *stated* its own totals to be, which
/// `application::supplier_invoice::import_supplier_invoice` needs to compare
/// against what [`crate::domain::supplier_invoice::service::SupplierInvoiceService::create_supplier_invoice`]
/// recomputes from the lines.
#[derive(Debug, Clone, PartialEq)]
pub struct ParsedSupplierInvoice {
    pub supplier_name: String,
    pub supplier_registration_number: Option<String>,
    pub supplier_vat_number: Option<String>,
    /// As issued by them, never allocated by us — same field, same
    /// reasoning as `SupplierInvoice::number`.
    pub number: String,
    pub issued_on: NaiveDate,
    pub due_on: Option<NaiveDate>,
    pub currency: String,
    pub lines: Vec<ParsedSupplierInvoiceLine>,
    /// As printed on the document. `None` when the CII profile omitted the
    /// summation tags outright (the schema allows it, even if real senders
    /// rarely do) — compared against the recomputed total only when
    /// present.
    pub stated_net_cents: Option<i32>,
    pub stated_gross_cents: Option<i32>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ParsedSupplierInvoiceLine {
    pub label: String,
    pub quantity: Decimal,
    pub unit: Option<String>,
    pub unit_price_cents: i32,
    /// Stored exactly as printed on the document — same reasoning as
    /// `SupplierInvoiceLine::line_total_cents`.
    pub line_total_cents: i32,
    pub vat_rate_basis_points: Option<i32>,
}

/// Why a file could not become a [`ParsedSupplierInvoice`]. Two variants,
/// not one, because #337 requires the two failure modes distinguishable: a
/// PDF with no Factur-X attachment at all (wrong file, or a supplier who
/// never enabled the profile) is a different problem than an attachment
/// that is present but whose XML does not parse (a malformed sender, or a
/// gap in our own reading of the schema) — a caller (and eventually a human
/// reviewing why an import vanished) needs to tell those apart.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum SupplierInvoiceParseError {
    #[error("could not read the Factur-X attachment from the PDF: {0}")]
    AttachmentExtraction(String),
    #[error("could not parse the Factur-X CII document: {0}")]
    XmlParsing(String),
}

/// Turns supplier-provided bytes into a [`ParsedSupplierInvoice`], or a
/// stated reason it could not — #337's binding rule: a file that cannot be
/// parsed is kept, with the reason, never silently discarded.
///
/// Sync, not `async fn` / no `impl Future`: both attachment extraction and
/// XML deserialization are CPU-bound work over bytes already fully in
/// memory (the caller has the whole upload before reaching this, never a
/// stream) — there is no I/O here for an `async fn` to buy anything.
#[cfg_attr(any(test, feature = "mock"), mockall::automock)]
pub trait SupplierInvoiceParser: Send {
    fn parse(&self, bytes: &[u8]) -> Result<ParsedSupplierInvoice, SupplierInvoiceParseError>;
}

/// The identifier used to tell one merchant from another before a
/// [`crate::Supplier`] row exists to point at (see that type's own doc
/// comment: recognising a supplier is out of this issue's scope). Whichever
/// of registration number, VAT number, or name the document actually
/// states, in that order — a free function, not a method, so both sides of
/// a duplicate comparison compute it identically: the incoming parsed
/// document here, and the stored row through
/// `PgSupplierInvoiceRepository::exists_with_duplicate_key`'s `COALESCE`,
/// which applies the exact same order.
pub fn supplier_identifier(
    registration_number: Option<&str>,
    vat_number: Option<&str>,
    name: &str,
) -> String {
    registration_number
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .or_else(|| vat_number.map(str::trim).filter(|value| !value.is_empty()))
        .unwrap_or(name)
        .to_owned()
}

/// Persists a [`SupplierInvoiceLineAllocation`] — a second, separate trait
/// rather than added methods on [`SupplierInvoiceRepository`], for two
/// reasons: that trait is owned by a sibling workstream (#337) running
/// concurrently against the same file, so adding to it would be a
/// guaranteed conflict; and an allocation is persisted through its own
/// table, with its own invariant enforced by its own database trigger, not
/// a variant of what the other trait already does.
#[cfg_attr(any(test, feature = "mock"), mockall::automock)]
pub trait SupplierInvoiceAllocationRepository: Send {
    fn insert(
        &mut self,
        allocation: &SupplierInvoiceLineAllocation,
    ) -> impl Future<Output = Result<SupplierInvoiceLineAllocation, CoreError>> + Send;

    /// The sum of every allocation already recorded against one line,
    /// `0` when there are none. What
    /// [`crate::domain::supplier_invoice::service::SupplierInvoiceService::allocate_line`]
    /// reads to refuse an overflow with a clear `CoreError::Conflict`
    /// before ever reaching the database's own trigger, which exists as
    /// the invariant's last line of defence, not its first.
    fn sum_allocated_for_line(
        &mut self,
        line_id: SupplierInvoiceLineId,
    ) -> impl Future<Output = Result<i32, CoreError>> + Send;

    /// Every allocation recorded against one project — the query #338 asks
    /// for on its own, independent of a full profitability report (which
    /// reads the same rows through `ProfitabilityRepository`'s own SQL
    /// join instead, for one round trip across every project rather than
    /// one call per project here).
    fn list_by_project(
        &mut self,
        project_id: ProjectId,
    ) -> impl Future<Output = Result<Vec<SupplierInvoiceLineAllocation>, CoreError>> + Send;

    /// Every allocation recorded against one line — what #339's full-replace
    /// `PUT .../allocations` reads before deciding what to delete and what
    /// to keep.
    fn list_by_line(
        &mut self,
        line_id: SupplierInvoiceLineId,
    ) -> impl Future<Output = Result<Vec<SupplierInvoiceLineAllocation>, CoreError>> + Send;

    /// Removes one allocation outright — never a soft delete. Unlike a
    /// supplier invoice itself, an allocation is our own bookkeeping, not
    /// somebody else's document; #339's full-replace semantics means a
    /// share dropped from the new list must stop counting immediately; a
    /// tombstone would keep costing a project #338 refuses to attribute it
    /// to anymore.
    fn delete(
        &mut self,
        id: SupplierInvoiceLineAllocationId,
    ) -> impl Future<Output = Result<(), CoreError>> + Send;

    /// Same rows as [`list_by_project`](Self::list_by_project), joined out
    /// to the line and invoice each one belongs to — #340's project screen
    /// reads this, not the bare list, since a cost with no way back to its
    /// invoice is not auditable.
    fn list_detailed_by_project(
        &mut self,
        project_id: ProjectId,
    ) -> impl Future<Output = Result<Vec<ProjectSupplierCostLine>, CoreError>> + Send;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn supplier_identifier_prefers_registration_number() {
        assert_eq!(
            supplier_identifier(Some("RCS 123"), Some("FR123"), "Point P"),
            "RCS 123"
        );
    }

    #[test]
    fn supplier_identifier_falls_back_to_vat_number() {
        assert_eq!(supplier_identifier(None, Some("FR123"), "Point P"), "FR123");
    }

    #[test]
    fn supplier_identifier_falls_back_to_name_when_neither_is_present() {
        assert_eq!(supplier_identifier(None, None, "Point P"), "Point P");
    }

    #[test]
    fn supplier_identifier_treats_a_blank_registration_number_as_absent() {
        assert_eq!(
            supplier_identifier(Some("   "), Some("FR123"), "Point P"),
            "FR123"
        );
    }
}
