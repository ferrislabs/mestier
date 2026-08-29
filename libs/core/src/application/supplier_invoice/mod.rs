use common::CoreError;
use mestier_macros::transactional;

use crate::{
    OrganizationId, ProjectId, ProjectSupplierCostLine, SupplierInvoice, SupplierInvoiceId,
    SupplierInvoiceLine, SupplierInvoiceLineAllocation, SupplierInvoiceLineId,
    SupplierInvoiceSource,
    application::MestierUseCase,
    domain::supplier_invoice::{
        commands::{
            AllocateSupplierInvoiceLineCommand, ConfirmSupplierInvoiceCommand,
            CreateSupplierInvoiceCommand, RejectSupplierInvoiceCommand,
            ReplaceSupplierInvoiceLineAllocationsCommand, SupplierInvoiceLineCommand,
            UpdateSupplierInvoiceNotesCommand,
        },
        ports::{
            SupplierInvoiceParseError, SupplierInvoiceParser, SupplierInvoiceRepository,
            supplier_identifier,
        },
        service::SupplierInvoiceService,
    },
};

/// What [`MestierUseCase::import_supplier_invoice`] (#337) hands back. Not
/// a plain `Result<SupplierInvoice, CoreError>`: a file that fails to parse
/// is an expected, legitimate outcome under the issue's own binding rule
/// ("kept, with the reason — never silently discarded"), not the same kind
/// of failure as losing the database connection mid-transaction — so it is
/// a value the caller must look at, not an error they might `?`-propagate
/// and forget.
///
/// A refused duplicate, by contrast, *is* a [`CoreError::Conflict`]: unlike
/// a parse failure it is not a new fact about a file the user is seeing for
/// the first time, it is this exact document (successfully read) already
/// on file — the same shape of refusal `SupplierInvoiceService::confirm`
/// already uses for "this document was already reviewed".
#[derive(Debug, Clone)]
pub enum ImportSupplierInvoiceOutcome {
    Created {
        // Boxed: `SupplierInvoice` carries its own `Vec<SupplierInvoiceLine>`
        // and VAT breakdown, dwarfing `ParseFailed`'s single `String` —
        // without this every `ImportSupplierInvoiceOutcome` would pay that
        // size regardless of which variant it holds.
        invoice: Box<SupplierInvoice>,
        /// `Some` when the document stated a total that disagrees with
        /// what `SupplierInvoiceService::create_supplier_invoice`
        /// recomputed from the lines — surfaced, never silently corrected
        /// (#337's other binding rule). Landing this on the outcome
        /// (rather than, say, refusing the import outright, or silently
        /// preferring one of the two totals) was the call made here: the
        /// document was still successfully read and its lines still add up
        /// internally, so refusing it entirely would throw away a real
        /// invoice over what is often a rounding difference in what the
        /// sender's own software printed; a human reviewing the `Received`
        /// document is better placed to judge that than an import step
        /// with no way to ask them.
        totals_mismatch: Option<TotalsMismatch>,
    },
    /// The file could not be parsed. Held here rather than turned into a
    /// bare `CoreError` and bubbled up: seeing this variant is what lets a
    /// handler show the reason instead of a generic failure. What this
    /// cannot yet do — persist the failed attempt itself for later review
    /// the way a successfully parsed document is — needs a repository/
    /// domain addition outside this issue's owned files; see the PR
    /// description's convergence-point request.
    ParseFailed { reason: SupplierInvoiceParseError },
}

/// The document's stated totals against what `create_supplier_invoice`
/// recomputed from the lines it just persisted.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TotalsMismatch {
    pub stated_net_cents: i32,
    pub recomputed_net_cents: i32,
    pub stated_gross_cents: i32,
    pub recomputed_gross_cents: i32,
}

impl MestierUseCase {
    /// Imports a Factur-X file as a `Received` supplier invoice (#337).
    /// `parser` is a normal argument, not a field on `self`: the port
    /// implementation is stateless (see `FacturXParser`'s own doc comment)
    /// and specific to this one call, unlike `supplier_invoice`/`emitter`,
    /// which the `#[transactional]` macro must inject on every method in
    /// this file because every one of them touches the database.
    #[transactional(supplier_invoice, emitter)]
    pub async fn import_supplier_invoice(
        &self,
        organization_id: OrganizationId,
        bytes: Vec<u8>,
        parser: &(impl SupplierInvoiceParser + Sync),
        // Already uploaded by the caller (#339's handler, reusing the
        // existing file storage service — see `libs/handlers-purchase`)
        // before this use case ever runs: uploading is a side effect
        // outside a domain transaction's job, and by the time bytes reach
        // here the original is already durable regardless of what this
        // call decides.
        source_file_key: String,
        source_file_mime_type: String,
    ) -> Result<ImportSupplierInvoiceOutcome, CoreError> {
        // Shadowed as `mut`: the macro injects an immutable binding, but the
        // duplicate check below needs `&mut self` on the repository ahead of
        // `SupplierInvoiceService::new` taking it by value — the same
        // binding, just reclaiming the mutability a plain move doesn't need.
        let mut supplier_invoice_repository = supplier_invoice_repository;

        let parsed = match parser.parse(&bytes) {
            Ok(parsed) => parsed,
            Err(reason) => {
                tracing::warn!(
                    organization_id = %organization_id.0,
                    error = %reason,
                    "supplier invoice import: file could not be parsed",
                );
                return Ok(ImportSupplierInvoiceOutcome::ParseFailed { reason });
            }
        };

        let identifier = supplier_identifier(
            parsed.supplier_registration_number.as_deref(),
            parsed.supplier_vat_number.as_deref(),
            &parsed.supplier_name,
        );

        let is_duplicate = supplier_invoice_repository
            .exists_with_duplicate_key(organization_id, &parsed.number, &identifier)
            .await?;
        if is_duplicate {
            return Err(CoreError::Conflict(format!(
                "supplier invoice {} from {} was already imported",
                parsed.number, parsed.supplier_name
            )));
        }

        let stated_net_cents = parsed.stated_net_cents;
        let stated_gross_cents = parsed.stated_gross_cents;

        let command = CreateSupplierInvoiceCommand {
            organization_id,
            supplier_name: parsed.supplier_name,
            supplier_registration_number: parsed.supplier_registration_number,
            supplier_vat_number: parsed.supplier_vat_number,
            number: parsed.number,
            issued_on: parsed.issued_on,
            due_on: parsed.due_on,
            source: SupplierInvoiceSource::FacturX,
            currency: parsed.currency,
            lines: parsed
                .lines
                .into_iter()
                .map(|line| SupplierInvoiceLineCommand {
                    label: line.label,
                    quantity: line.quantity,
                    unit: line.unit,
                    unit_price_cents: line.unit_price_cents,
                    line_total_cents: line.line_total_cents,
                    vat_rate_basis_points: line.vat_rate_basis_points,
                })
                .collect(),
            source_file_key: Some(source_file_key),
            source_file_mime_type: Some(source_file_mime_type),
        };

        let mut service = SupplierInvoiceService::new(supplier_invoice_repository, emitter);
        let invoice = service.create_supplier_invoice(command).await?;
        let totals_mismatch =
            detect_totals_mismatch(stated_net_cents, stated_gross_cents, &invoice);

        Ok(ImportSupplierInvoiceOutcome::Created {
            invoice: Box::new(invoice),
            totals_mismatch,
        })
    }

    #[transactional(supplier_invoice, emitter)]
    pub async fn create_supplier_invoice(
        &self,
        command: CreateSupplierInvoiceCommand,
    ) -> Result<SupplierInvoice, CoreError> {
        let mut service = SupplierInvoiceService::new(supplier_invoice_repository, emitter);
        service.create_supplier_invoice(command).await
    }

    #[transactional(supplier_invoice, emitter)]
    pub async fn get_supplier_invoice(
        &self,
        id: SupplierInvoiceId,
    ) -> Result<SupplierInvoice, CoreError> {
        let mut service = SupplierInvoiceService::new(supplier_invoice_repository, emitter);
        service.get_supplier_invoice(id).await
    }

    #[transactional(supplier_invoice, emitter)]
    pub async fn list_supplier_invoices(
        &self,
        organization_id: OrganizationId,
        limit: u64,
        offset: u64,
    ) -> Result<(Vec<SupplierInvoice>, u64), CoreError> {
        let mut service = SupplierInvoiceService::new(supplier_invoice_repository, emitter);
        service
            .list_supplier_invoices(organization_id, limit, offset)
            .await
    }

    #[transactional(supplier_invoice, emitter)]
    pub async fn confirm_supplier_invoice(
        &self,
        command: ConfirmSupplierInvoiceCommand,
    ) -> Result<SupplierInvoice, CoreError> {
        let mut service = SupplierInvoiceService::new(supplier_invoice_repository, emitter);
        service.confirm(command).await
    }

    #[transactional(supplier_invoice, emitter)]
    pub async fn reject_supplier_invoice(
        &self,
        command: RejectSupplierInvoiceCommand,
    ) -> Result<SupplierInvoice, CoreError> {
        let mut service = SupplierInvoiceService::new(supplier_invoice_repository, emitter);
        service.reject(command).await
    }

    /// #339's metadata-only `PATCH` — `notes` alone, never the document's
    /// own fields.
    #[transactional(supplier_invoice, emitter)]
    pub async fn update_supplier_invoice_notes(
        &self,
        command: UpdateSupplierInvoiceNotesCommand,
    ) -> Result<SupplierInvoice, CoreError> {
        let mut service = SupplierInvoiceService::new(supplier_invoice_repository, emitter);
        service.update_notes(command).await
    }

    /// Resolves a bare line id to the line itself — #339's handler needs
    /// this before it can build a
    /// [`ReplaceSupplierInvoiceLineAllocationsCommand`], since the route
    /// (CLAUDE.md: bare ids derive their organization from the loaded row)
    /// carries only `supplier_invoice_line_id`.
    #[transactional(supplier_invoice)]
    pub async fn find_supplier_invoice_line(
        &self,
        id: SupplierInvoiceLineId,
    ) -> Result<Option<SupplierInvoiceLine>, CoreError> {
        // Shadowed as `mut` for the same reason `import_supplier_invoice`
        // does: the macro injects an immutable binding.
        let mut supplier_invoice_repository = supplier_invoice_repository;
        supplier_invoice_repository.find_line_by_id(id).await
    }

    /// #339's full-replace `PUT .../supplier-invoice-lines/{line_id}/allocations`.
    #[transactional(supplier_invoice, supplier_invoice_allocation, emitter)]
    pub async fn replace_supplier_invoice_line_allocations(
        &self,
        command: ReplaceSupplierInvoiceLineAllocationsCommand,
    ) -> Result<Vec<SupplierInvoiceLineAllocation>, CoreError> {
        let mut service = SupplierInvoiceService::new(supplier_invoice_repository, emitter);
        service
            .replace_line_allocations(command, supplier_invoice_allocation_repository)
            .await
    }

    /// #338: attributes part (or all) of a confirmed-or-not line's cost to
    /// a project. `supplier_invoice_allocation` names a second, sibling
    /// repository binding — additive, resolved through the same
    /// `RepoFor<SupplierInvoiceAllocation>` registry marker as every other
    /// domain, and does not touch `supplier_invoice`'s own binding.
    #[transactional(supplier_invoice, supplier_invoice_allocation, emitter)]
    pub async fn allocate_supplier_invoice_line(
        &self,
        command: AllocateSupplierInvoiceLineCommand,
    ) -> Result<SupplierInvoiceLineAllocation, CoreError> {
        let mut service = SupplierInvoiceService::new(supplier_invoice_repository, emitter);
        service
            .allocate_line(command, supplier_invoice_allocation_repository)
            .await
    }

    /// The net sum of every allocation recorded against one project so far
    /// — see the doc comment on
    /// `SupplierInvoiceService::allocated_cost_for_project` for how this
    /// differs from what a profitability report states.
    #[transactional(supplier_invoice, supplier_invoice_allocation, emitter)]
    pub async fn allocated_supplier_cost_for_project(
        &self,
        project_id: ProjectId,
    ) -> Result<i64, CoreError> {
        let mut service = SupplierInvoiceService::new(supplier_invoice_repository, emitter);
        service
            .allocated_cost_for_project(project_id, supplier_invoice_allocation_repository)
            .await
    }

    /// #340's itemized read: same allocations as
    /// [`allocated_supplier_cost_for_project`](Self::allocated_supplier_cost_for_project),
    /// each carrying the invoice it belongs to.
    #[transactional(supplier_invoice, supplier_invoice_allocation, emitter)]
    pub async fn project_supplier_cost_lines(
        &self,
        project_id: ProjectId,
    ) -> Result<Vec<ProjectSupplierCostLine>, CoreError> {
        let mut service = SupplierInvoiceService::new(supplier_invoice_repository, emitter);
        service
            .project_supplier_cost_lines(project_id, supplier_invoice_allocation_repository)
            .await
    }
}

/// Pulled out of `import_supplier_invoice` as a pure function on purpose:
/// that method is `#[transactional]`, so exercising it needs a real
/// Postgres transaction (see this crate's `--ignored` integration suite);
/// this comparison — #337's "surfaced, not corrected" rule — has no
/// business needing one, and a format parser or a totals comparison with a
/// database dependency is a rule nobody actually runs in CI.
fn detect_totals_mismatch(
    stated_net_cents: Option<i32>,
    stated_gross_cents: Option<i32>,
    invoice: &SupplierInvoice,
) -> Option<TotalsMismatch> {
    let stated_net_cents = stated_net_cents?;
    let stated_gross_cents = stated_gross_cents?;

    if stated_net_cents == invoice.net_cents && stated_gross_cents == invoice.gross_cents {
        return None;
    }

    Some(TotalsMismatch {
        stated_net_cents,
        recomputed_net_cents: invoice.net_cents,
        stated_gross_cents,
        recomputed_gross_cents: invoice.gross_cents,
    })
}

#[cfg(test)]
mod tests {
    use chrono::{NaiveDate, Utc};

    use super::*;
    use crate::{SupplierInvoiceId, SupplierInvoiceStatus};

    fn invoice_with_totals(net_cents: i32, gross_cents: i32) -> SupplierInvoice {
        let now = Utc::now();
        SupplierInvoice {
            id: SupplierInvoiceId(uuid::Uuid::new_v4()),
            organization_id: OrganizationId(uuid::Uuid::new_v4()),
            supplier_id: None,
            supplier_name: "Point P".to_owned(),
            supplier_registration_number: None,
            supplier_vat_number: None,
            number: "F-2026-4471".to_owned(),
            issued_on: NaiveDate::from_ymd_opt(2026, 8, 20).unwrap(),
            due_on: None,
            received_at: now,
            source: SupplierInvoiceSource::FacturX,
            status: SupplierInvoiceStatus::Received,
            currency: "EUR".to_owned(),
            source_file_key: None,
            source_file_mime_type: None,
            notes: None,
            net_cents,
            vat_breakdown: vec![],
            gross_cents,
            lines: vec![],
            deleted_at: None,
            created_at: now,
            updated_at: now,
        }
    }

    #[test]
    fn no_mismatch_when_the_document_stated_no_totals_at_all() {
        let invoice = invoice_with_totals(45_000, 54_000);

        assert_eq!(detect_totals_mismatch(None, None, &invoice), None);
    }

    #[test]
    fn no_mismatch_when_stated_and_recomputed_totals_agree() {
        let invoice = invoice_with_totals(45_000, 54_000);

        assert_eq!(
            detect_totals_mismatch(Some(45_000), Some(54_000), &invoice),
            None
        );
    }

    #[test]
    fn surfaces_a_mismatch_without_correcting_either_side() {
        let invoice = invoice_with_totals(45_000, 54_000);

        let mismatch = detect_totals_mismatch(Some(44_000), Some(54_000), &invoice).unwrap();

        assert_eq!(
            mismatch,
            TotalsMismatch {
                stated_net_cents: 44_000,
                recomputed_net_cents: 45_000,
                stated_gross_cents: 54_000,
                recomputed_gross_cents: 54_000,
            }
        );
    }

    #[test]
    fn a_mismatch_on_gross_alone_is_still_surfaced() {
        let invoice = invoice_with_totals(45_000, 54_000);

        assert!(detect_totals_mismatch(Some(45_000), Some(50_000), &invoice).is_some());
    }
}
