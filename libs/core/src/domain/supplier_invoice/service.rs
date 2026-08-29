use std::collections::BTreeMap;

use chrono::Utc;
use common::{CoreError, generate_uuid_v7};
use events::EventEmitter;
use rust_decimal::Decimal;

use crate::{
    OrganizationId, ProjectId, SupplierInvoice, SupplierInvoiceId, SupplierInvoiceLine,
    SupplierInvoiceLineAllocation, SupplierInvoiceLineAllocationId, SupplierInvoiceLineId,
    SupplierInvoiceReview, SupplierInvoiceStatus, SupplierInvoiceVatBreakdownLine,
    domain::supplier_invoice::{
        commands::{
            AllocateSupplierInvoiceLineCommand, ConfirmSupplierInvoiceCommand,
            CreateSupplierInvoiceCommand, RejectSupplierInvoiceCommand,
            ReplaceSupplierInvoiceLineAllocationsCommand, SupplierInvoiceLineCommand,
            UpdateSupplierInvoiceNotesCommand,
        },
        events::{
            SupplierInvoiceLineAllocated, SupplierInvoiceReceived, SupplierInvoiceTransitioned,
        },
        ports::{SupplierInvoiceAllocationRepository, SupplierInvoiceRepository},
    },
};

pub struct SupplierInvoiceService<R, E>
where
    R: SupplierInvoiceRepository,
    E: EventEmitter,
{
    repo: R,
    emitter: E,
}

impl<R, E> SupplierInvoiceService<R, E>
where
    R: SupplierInvoiceRepository,
    E: EventEmitter,
{
    pub fn new(repo: R, emitter: E) -> Self {
        Self { repo, emitter }
    }

    pub async fn create_supplier_invoice(
        &mut self,
        command: CreateSupplierInvoiceCommand,
    ) -> Result<SupplierInvoice, CoreError> {
        validate_currency(&command.currency)?;

        let now = Utc::now();
        let invoice_id = SupplierInvoiceId(generate_uuid_v7());
        let lines = build_lines(command.organization_id, invoice_id, command.lines, now)?;
        let totals = calculate_totals(&lines)?;

        let invoice = SupplierInvoice {
            id: invoice_id,
            organization_id: command.organization_id,
            supplier_id: None,
            supplier_name: non_blank(command.supplier_name, "supplier name")?,
            supplier_registration_number: non_blank_option(command.supplier_registration_number)?,
            supplier_vat_number: non_blank_option(command.supplier_vat_number)?,
            number: non_blank(command.number, "number")?,
            issued_on: command.issued_on,
            due_on: command.due_on,
            received_at: now,
            source: command.source,
            status: SupplierInvoiceStatus::Received,
            currency: command.currency.to_uppercase(),
            source_file_key: command.source_file_key,
            source_file_mime_type: command.source_file_mime_type,
            notes: None,
            net_cents: totals.net_cents,
            vat_breakdown: totals.vat_breakdown,
            gross_cents: totals.gross_cents,
            lines,
            deleted_at: None,
            created_at: now,
            updated_at: now,
        };

        let created = self.repo.insert(&invoice).await?;

        self.emitter.emit(
            created.organization_id,
            &SupplierInvoiceReceived {
                invoice: created.clone(),
            },
        )?;

        Ok(created)
    }

    pub async fn get_supplier_invoice(
        &mut self,
        id: SupplierInvoiceId,
    ) -> Result<SupplierInvoice, CoreError> {
        self.repo.find_by_id(id).await?.ok_or(CoreError::NotFound)
    }

    pub async fn list_supplier_invoices(
        &mut self,
        organization_id: OrganizationId,
        limit: u64,
        offset: u64,
    ) -> Result<(Vec<SupplierInvoice>, u64), CoreError> {
        self.repo
            .list_by_organization(organization_id, limit, offset)
            .await
    }

    /// Accepts a `Received` document as-is. Refused on anything else — a
    /// document is reviewed exactly once, the same rule
    /// `InvoiceService::cancel_invoice` enforces one directory over.
    pub async fn confirm(
        &mut self,
        command: ConfirmSupplierInvoiceCommand,
    ) -> Result<SupplierInvoice, CoreError> {
        self.transition(command.id, SupplierInvoiceStatus::Confirmed, command.notes)
            .await
    }

    /// Refuses a `Received` document. Same one-shot rule as [`Self::confirm`].
    pub async fn reject(
        &mut self,
        command: RejectSupplierInvoiceCommand,
    ) -> Result<SupplierInvoice, CoreError> {
        self.transition(command.id, SupplierInvoiceStatus::Rejected, command.notes)
            .await
    }

    /// Edits `notes` alone — #339's metadata-only `PATCH`. No status guard,
    /// unlike `confirm`/`reject`: a note is our own commentary, editable
    /// regardless of where the document's own review stands, never the
    /// one-shot act a status transition is.
    pub async fn update_notes(
        &mut self,
        command: UpdateSupplierInvoiceNotesCommand,
    ) -> Result<SupplierInvoice, CoreError> {
        let existing = self.get_supplier_invoice(command.id).await?;
        let mut review = SupplierInvoiceReview::new(existing);
        review.set_notes(non_blank_option(command.notes)?);
        review.touch(Utc::now());

        self.repo.update_review(&review).await
    }

    /// Attributes part (or all) of a line's own cost to a project — #338.
    ///
    /// `allocation_repo` is a method-level generic, the same shape
    /// `InvoiceService::create_invoice` takes its `organization_repository`
    /// argument: this service is not generic over a second repository type
    /// at the struct level, because every other constructor of
    /// [`SupplierInvoiceService`] — five of them, across this file's own
    /// tests and every use case in `application::supplier_invoice` — would
    /// otherwise need a matching type argument for a repository they never
    /// touch.
    ///
    /// Refuses an overflow itself, with a `CoreError::Conflict` naming the
    /// figures, before ever reaching the database: the trigger on
    /// `supplier_invoice_line_allocations` enforces the same bound and is
    /// what makes it true regardless of this check, but a bare constraint
    /// violation surfacing through `map_sqlx_error` would tell a caller far
    /// less than this can.
    pub async fn allocate_line<A>(
        &mut self,
        command: AllocateSupplierInvoiceLineCommand,
        mut allocation_repo: A,
    ) -> Result<SupplierInvoiceLineAllocation, CoreError>
    where
        A: SupplierInvoiceAllocationRepository,
    {
        if command.amount_cents == 0 {
            return Err(CoreError::Conflict(
                "an allocation cannot be zero".to_owned(),
            ));
        }

        let invoice = self
            .get_supplier_invoice(command.supplier_invoice_id)
            .await?;
        if invoice.organization_id != command.organization_id {
            return Err(CoreError::NotFound);
        }

        let line = invoice
            .lines
            .iter()
            .find(|line| line.id == command.supplier_invoice_line_id)
            .ok_or(CoreError::NotFound)?;

        refuse_sign_mismatch(line, command.amount_cents)?;

        let already_allocated = allocation_repo.sum_allocated_for_line(line.id).await?;
        let new_total = i64::from(already_allocated) + i64::from(command.amount_cents);
        refuse_if_overflows(line, new_total)?;

        let now = Utc::now();
        let allocation = SupplierInvoiceLineAllocation {
            id: SupplierInvoiceLineAllocationId(generate_uuid_v7()),
            organization_id: command.organization_id,
            supplier_invoice_line_id: line.id,
            project_id: command.project_id,
            amount_cents: command.amount_cents,
            created_at: now,
            updated_at: now,
        };

        let created = allocation_repo.insert(&allocation).await?;

        self.emitter.emit(
            command.organization_id,
            &SupplierInvoiceLineAllocated {
                allocation: created.clone(),
            },
        )?;

        Ok(created)
    }

    /// The net sum of every allocation recorded against one project so far
    /// — the query #338 asks for on its own, independent of a full
    /// profitability report. Deliberately *not* the same figure a
    /// profitability report states: this is a plain net sum across every
    /// invoice regardless of status or period, useful for a line's own
    /// screen ("X of Y allocated"), while the report folds in only
    /// `Confirmed` invoices, only the report's own period, and grosses the
    /// figure up when the organization cannot recover VAT (see
    /// `profitability::service::build_report`).
    pub async fn allocated_cost_for_project<A>(
        &mut self,
        project_id: ProjectId,
        mut allocation_repo: A,
    ) -> Result<i64, CoreError>
    where
        A: SupplierInvoiceAllocationRepository,
    {
        let allocations = allocation_repo.list_by_project(project_id).await?;

        Ok(allocations
            .iter()
            .map(|allocation| i64::from(allocation.amount_cents))
            .sum())
    }

    /// Full-replace of one line's allocations — #339's `PUT
    /// .../supplier-invoice-lines/{line_id}/allocations`, mirroring
    /// `Task::assignments`'s own contract: the body is the complete list
    /// for the line, not a delta.
    ///
    /// Diffs against what is already recorded rather than deleting
    /// everything and reinserting the new set: a share that survives the
    /// replace unchanged keeps its own `id`/`created_at`, which matters to
    /// anything that already refers to that allocation by id (the events
    /// this emits, an audit trail reading `created_at`). Only shares no
    /// longer present are deleted; only genuinely new ones are inserted.
    pub async fn replace_line_allocations<A>(
        &mut self,
        command: ReplaceSupplierInvoiceLineAllocationsCommand,
        mut allocation_repo: A,
    ) -> Result<Vec<SupplierInvoiceLineAllocation>, CoreError>
    where
        A: SupplierInvoiceAllocationRepository,
    {
        if command
            .allocations
            .iter()
            .any(|share| share.amount_cents == 0)
        {
            return Err(CoreError::Conflict(
                "an allocation cannot be zero".to_owned(),
            ));
        }

        let invoice = self
            .get_supplier_invoice(command.supplier_invoice_id)
            .await?;
        if invoice.organization_id != command.organization_id {
            return Err(CoreError::NotFound);
        }

        let line = invoice
            .lines
            .iter()
            .find(|line| line.id == command.supplier_invoice_line_id)
            .ok_or(CoreError::NotFound)?;

        for share in &command.allocations {
            refuse_sign_mismatch(line, share.amount_cents)?;
        }

        let candidate_total: i64 = command
            .allocations
            .iter()
            .map(|share| i64::from(share.amount_cents))
            .sum();
        refuse_if_overflows(line, candidate_total)?;

        let mut remaining_existing = allocation_repo.list_by_line(line.id).await?;
        let now = Utc::now();
        let mut result = Vec::with_capacity(command.allocations.len());

        for share in command.allocations {
            // Matched by project alone rather than by amount too: a line
            // rarely carries more than one share per project, and a caller
            // who really did mean to split one project's share into two
            // would already be misusing "the complete list for this line"
            // as something other than a flat set.
            let previous = remaining_existing
                .iter()
                .position(|allocation| allocation.project_id == share.project_id)
                .map(|index| remaining_existing.remove(index));

            // A byte-identical share (same project, same amount) is left
            // exactly as it was — no repository call, no new event. There
            // is no `update` on this port: an amount that did change is a
            // delete-then-recreate, the same "not sent as it was, so not
            // kept as it was" rule applied one level down from a whole
            // dropped share.
            if let Some(previous) = &previous
                && previous.amount_cents == share.amount_cents
            {
                result.push(previous.clone());
                continue;
            }
            if let Some(previous) = previous {
                allocation_repo.delete(previous.id).await?;
            }

            let allocation = SupplierInvoiceLineAllocation {
                id: SupplierInvoiceLineAllocationId(generate_uuid_v7()),
                organization_id: command.organization_id,
                supplier_invoice_line_id: line.id,
                project_id: share.project_id,
                amount_cents: share.amount_cents,
                created_at: now,
                updated_at: now,
            };
            let created = allocation_repo.insert(&allocation).await?;
            self.emitter.emit(
                command.organization_id,
                &SupplierInvoiceLineAllocated {
                    allocation: created.clone(),
                },
            )?;
            result.push(created);
        }

        // Whatever is left in `remaining_existing` had no matching share in
        // the new list — the same "not sent, so not wanted" rule
        // `Task::assignments` applies to a dropped assignee.
        for stale in remaining_existing {
            allocation_repo.delete(stale.id).await?;
        }

        Ok(result)
    }

    async fn transition(
        &mut self,
        id: SupplierInvoiceId,
        to: SupplierInvoiceStatus,
        notes: Option<String>,
    ) -> Result<SupplierInvoice, CoreError> {
        let existing = self.get_supplier_invoice(id).await?;
        if existing.status != SupplierInvoiceStatus::Received {
            return Err(CoreError::Conflict(format!(
                "supplier invoice {} is {} and cannot be reviewed again; only a received \
                 document can be",
                existing.id, existing.status
            )));
        }

        let from = existing.status;
        let mut review = SupplierInvoiceReview::new(existing);
        review.set_status(to);
        if notes.is_some() {
            review.set_notes(non_blank_option(notes)?);
        }
        review.touch(Utc::now());

        let updated = self.repo.update_review(&review).await?;

        self.emitter.emit(
            updated.organization_id,
            &SupplierInvoiceTransitioned {
                invoice: updated.clone(),
                from,
            },
        )?;

        Ok(updated)
    }
}

fn non_blank(value: String, field: &str) -> Result<String, CoreError> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(CoreError::Conflict(format!("{field} cannot be empty")));
    }

    Ok(trimmed.to_owned())
}

fn non_blank_option(value: Option<String>) -> Result<Option<String>, CoreError> {
    value
        .map(|value| {
            let trimmed = value.trim();
            if trimmed.is_empty() {
                return Err(CoreError::Conflict(
                    "value cannot be blank when present".to_owned(),
                ));
            }

            Ok(trimmed.to_owned())
        })
        .transpose()
}

fn validate_currency(currency: &str) -> Result<(), CoreError> {
    let trimmed = currency.trim();
    if trimmed.len() != 3 || !trimmed.chars().all(|c| c.is_ascii_alphabetic()) {
        return Err(CoreError::Conflict(
            "supplier invoice currency must be a 3-letter ISO code".to_owned(),
        ));
    }

    Ok(())
}

fn build_lines(
    organization_id: OrganizationId,
    invoice_id: SupplierInvoiceId,
    commands: Vec<SupplierInvoiceLineCommand>,
    now: chrono::DateTime<Utc>,
) -> Result<Vec<SupplierInvoiceLine>, CoreError> {
    if commands.is_empty() {
        return Err(CoreError::Conflict(
            "a supplier invoice must have at least one line".to_owned(),
        ));
    }

    commands
        .into_iter()
        .enumerate()
        .map(|(position, command)| {
            build_line(organization_id, invoice_id, command, position as i32, now)
        })
        .collect()
}

fn build_line(
    organization_id: OrganizationId,
    invoice_id: SupplierInvoiceId,
    command: SupplierInvoiceLineCommand,
    position: i32,
    now: chrono::DateTime<Utc>,
) -> Result<SupplierInvoiceLine, CoreError> {
    validate_line(&command)?;

    Ok(SupplierInvoiceLine {
        id: SupplierInvoiceLineId(generate_uuid_v7()),
        organization_id,
        supplier_invoice_id: invoice_id,
        label: command.label.trim().to_owned(),
        quantity: command.quantity,
        unit: non_blank_option(command.unit)?,
        unit_price_cents: command.unit_price_cents,
        line_total_cents: command.line_total_cents,
        vat_rate_basis_points: command.vat_rate_basis_points,
        position,
        deleted_at: None,
        created_at: now,
        updated_at: now,
    })
}

fn validate_line(command: &SupplierInvoiceLineCommand) -> Result<(), CoreError> {
    if command.label.trim().is_empty() {
        return Err(CoreError::Conflict(
            "supplier invoice line label cannot be empty".to_owned(),
        ));
    }

    // Not `> 0`: a credit/rebate line from the supplier is a legitimate
    // line on a received document, not a state we get to refuse the way
    // `InvoiceLine` refuses one on a document we author ourselves. Only an
    // exactly-zero quantity is meaningless.
    if command.quantity == Decimal::ZERO {
        return Err(CoreError::Conflict(
            "supplier invoice line quantity cannot be zero".to_owned(),
        ));
    }

    if command
        .vat_rate_basis_points
        .is_some_and(|rate_bp| !(0..=10_000).contains(&rate_bp))
    {
        return Err(CoreError::Conflict(
            "supplier invoice line vat rate must be between 0 and 10000 basis points".to_owned(),
        ));
    }

    Ok(())
}

/// Same sign as the line: a credit/rebate line can only be allocated as a
/// credit, never flipped into a positive cost by the act of allocating it.
/// Shared by [`SupplierInvoiceService::allocate_line`] and
/// [`SupplierInvoiceService::replace_line_allocations`] — the same rule,
/// checked once from either entry point.
fn refuse_sign_mismatch(line: &SupplierInvoiceLine, amount_cents: i32) -> Result<(), CoreError> {
    if (line.line_total_cents >= 0) != (amount_cents >= 0) {
        return Err(CoreError::Conflict(format!(
            "allocation {} cents must have the same sign as line {}'s total of {} cents",
            amount_cents, line.id, line.line_total_cents
        )));
    }

    Ok(())
}

/// Refuses a candidate allocation total that would push a line past its own
/// `line_total_cents`, in either direction (a negative/credit line overflows
/// downward, a positive one upward) — the same domain-level check the
/// database's own trigger enforces as its last line of defence, not its
/// first. Shared by the same two callers as [`refuse_sign_mismatch`].
fn refuse_if_overflows(line: &SupplierInvoiceLine, candidate_total: i64) -> Result<(), CoreError> {
    let overflows = if line.line_total_cents >= 0 {
        candidate_total > i64::from(line.line_total_cents)
    } else {
        candidate_total < i64::from(line.line_total_cents)
    };
    if overflows {
        return Err(CoreError::Conflict(format!(
            "allocating to line {} would bring its allocations to {} cents, beyond its {} cent \
             total",
            line.id, candidate_total, line.line_total_cents
        )));
    }

    Ok(())
}

pub(crate) struct SupplierInvoiceTotals {
    pub net_cents: i32,
    pub vat_breakdown: Vec<SupplierInvoiceVatBreakdownLine>,
    pub gross_cents: i32,
}

/// Net is the sum of what the supplier printed on each line — never
/// recomputed from `quantity * unit_price_cents`, the same reasoning as
/// `SupplierInvoiceLine::line_total_cents` itself. VAT is our own reading,
/// broken down per rate present on the lines, the same rounding rule
/// `invoice::service::calculate_totals` uses: rounded per line then summed
/// per rate, not summed then rounded.
pub(crate) fn calculate_totals(
    lines: &[SupplierInvoiceLine],
) -> Result<SupplierInvoiceTotals, CoreError> {
    let net_cents = lines.iter().try_fold(0_i64, |sum, line| {
        sum.checked_add(i64::from(line.line_total_cents))
            .ok_or_else(|| {
                CoreError::Conflict("supplier invoice total is outside supported bounds".to_owned())
            })
    })?;

    let mut vat_by_rate: BTreeMap<i32, i64> = BTreeMap::new();
    let mut vat_total: i64 = 0;

    for line in lines {
        let Some(rate_bp) = line.vat_rate_basis_points else {
            continue;
        };
        let line_vat = div_round_half_even(
            i64::from(line.line_total_cents) * i64::from(rate_bp),
            10_000,
        );

        *vat_by_rate.entry(rate_bp).or_insert(0) += line_vat;
        vat_total = vat_total.checked_add(line_vat).ok_or_else(|| {
            CoreError::Conflict("supplier invoice VAT total is outside supported bounds".to_owned())
        })?;
    }

    let vat_breakdown = vat_by_rate
        .into_iter()
        .map(|(rate_bp, vat_cents)| {
            i32::try_from(vat_cents)
                .map(|vat_cents| SupplierInvoiceVatBreakdownLine { rate_bp, vat_cents })
                .map_err(|_| {
                    CoreError::Conflict(
                        "supplier invoice VAT total is outside supported bounds".to_owned(),
                    )
                })
        })
        .collect::<Result<Vec<_>, CoreError>>()?;

    let net_cents = i32::try_from(net_cents).map_err(|_| {
        CoreError::Conflict("supplier invoice total is outside supported bounds".to_owned())
    })?;
    let gross_cents = i64::from(net_cents)
        .checked_add(vat_total)
        .and_then(|total| i32::try_from(total).ok())
        .ok_or_else(|| {
            CoreError::Conflict("supplier invoice total is outside supported bounds".to_owned())
        })?;

    Ok(SupplierInvoiceTotals {
        net_cents,
        vat_breakdown,
        gross_cents,
    })
}

/// Divides, rounding a half to the even neighbour. Duplicated from
/// `invoice::service::div_round_half_even` — same reasoning: two
/// occurrences is not yet a pattern worth sharing.
fn div_round_half_even(numerator: i64, denominator: i64) -> i64 {
    let quotient = numerator / denominator;
    let doubled_remainder = (numerator % denominator) * 2;

    if doubled_remainder > denominator {
        return quotient + 1;
    }
    if doubled_remainder < denominator {
        return quotient;
    }

    if quotient % 2 == 0 {
        quotient
    } else {
        quotient + 1
    }
}

#[cfg(test)]
mod tests {
    use chrono::NaiveDate;
    use events::testing::RecordingEmitter;
    use mockall::predicate::always;

    use super::*;
    use crate::{
        LineAllocationShare, MockSupplierInvoiceAllocationRepository,
        MockSupplierInvoiceRepository, OrganizationId, ProjectId, SupplierInvoiceSource,
    };

    fn line(
        line_total_cents: i32,
        vat_rate_basis_points: Option<i32>,
    ) -> SupplierInvoiceLineCommand {
        SupplierInvoiceLineCommand {
            label: "Plaques de plâtre".to_owned(),
            quantity: Decimal::from(10),
            unit: Some("u".to_owned()),
            unit_price_cents: line_total_cents / 10,
            line_total_cents,
            vat_rate_basis_points,
        }
    }

    fn create_command(lines: Vec<SupplierInvoiceLineCommand>) -> CreateSupplierInvoiceCommand {
        CreateSupplierInvoiceCommand {
            organization_id: OrganizationId(uuid::Uuid::new_v4()),
            supplier_name: "Point P".to_owned(),
            supplier_registration_number: None,
            supplier_vat_number: None,
            number: "F-2026-4471".to_owned(),
            issued_on: NaiveDate::from_ymd_opt(2026, 8, 20).unwrap(),
            due_on: None,
            source: SupplierInvoiceSource::Manual,
            currency: "eur".to_owned(),
            lines,
            source_file_key: None,
            source_file_mime_type: None,
        }
    }

    #[tokio::test]
    async fn create_supplier_invoice_computes_totals_from_line_totals_not_quantity_times_price() {
        let mut repo = MockSupplierInvoiceRepository::new();
        repo.expect_insert()
            .withf(|invoice| invoice.net_cents == 45_000 && invoice.gross_cents == 54_000)
            .returning(|invoice| {
                let invoice = invoice.clone();
                Box::pin(async move { Ok(invoice) })
            });

        let emitter = RecordingEmitter::new();
        let mut service = SupplierInvoiceService::new(repo, &emitter);
        let created = service
            .create_supplier_invoice(create_command(vec![line(45_000, Some(2000))]))
            .await
            .unwrap();

        assert_eq!(created.status, SupplierInvoiceStatus::Received);
        assert_eq!(created.currency, "EUR");
        assert_eq!(created.net_cents, 45_000);
        assert_eq!(
            created.vat_breakdown,
            vec![SupplierInvoiceVatBreakdownLine {
                rate_bp: 2000,
                vat_cents: 9_000,
            }]
        );
        assert_eq!(created.gross_cents, 54_000);
        assert_eq!(emitter.names(), vec!["supplier_invoice.received"]);
    }

    #[tokio::test]
    async fn create_supplier_invoice_refuses_an_empty_line_list() {
        let repo = MockSupplierInvoiceRepository::new();
        let emitter = RecordingEmitter::new();
        let mut service = SupplierInvoiceService::new(repo, &emitter);

        let result = service
            .create_supplier_invoice(create_command(vec![]))
            .await;

        assert!(result.is_err());
        assert!(emitter.names().is_empty());
    }

    #[tokio::test]
    async fn confirm_refuses_a_document_that_is_not_received() {
        let mut repo = MockSupplierInvoiceRepository::new();
        repo.expect_find_by_id()
            .with(always())
            .returning(|id| Box::pin(async move { Ok(Some(confirmed_fixture(id))) }));

        let emitter = RecordingEmitter::new();
        let mut service = SupplierInvoiceService::new(repo, &emitter);
        let result = service
            .confirm(ConfirmSupplierInvoiceCommand {
                id: SupplierInvoiceId(uuid::Uuid::new_v4()),
                notes: None,
            })
            .await;

        assert!(matches!(result, Err(CoreError::Conflict(_))));
    }

    fn confirmed_fixture(id: SupplierInvoiceId) -> SupplierInvoice {
        let now = Utc::now();
        SupplierInvoice {
            id,
            organization_id: OrganizationId(uuid::Uuid::new_v4()),
            supplier_id: None,
            supplier_name: "Point P".to_owned(),
            supplier_registration_number: None,
            supplier_vat_number: None,
            number: "F-2026-4471".to_owned(),
            issued_on: NaiveDate::from_ymd_opt(2026, 8, 20).unwrap(),
            due_on: None,
            received_at: now,
            source: SupplierInvoiceSource::Manual,
            status: SupplierInvoiceStatus::Confirmed,
            currency: "EUR".to_owned(),
            source_file_key: None,
            source_file_mime_type: None,
            notes: None,
            net_cents: 45_000,
            vat_breakdown: vec![],
            gross_cents: 45_000,
            lines: vec![],
            deleted_at: None,
            created_at: now,
            updated_at: now,
        }
    }

    /// A `Received` invoice carrying one line, for the `allocate_line` tests
    /// below — `Received` on purpose: allocating is allowed before review,
    /// see the doc comment on `AllocateSupplierInvoiceLineCommand`.
    fn invoice_with_line(
        organization_id: OrganizationId,
        invoice_id: SupplierInvoiceId,
        line_id: SupplierInvoiceLineId,
        line_total_cents: i32,
    ) -> SupplierInvoice {
        let now = Utc::now();
        SupplierInvoice {
            id: invoice_id,
            organization_id,
            supplier_id: None,
            supplier_name: "Point P".to_owned(),
            supplier_registration_number: None,
            supplier_vat_number: None,
            number: "F-2026-4471".to_owned(),
            issued_on: NaiveDate::from_ymd_opt(2026, 8, 20).unwrap(),
            due_on: None,
            received_at: now,
            source: SupplierInvoiceSource::Manual,
            status: SupplierInvoiceStatus::Received,
            currency: "EUR".to_owned(),
            source_file_key: None,
            source_file_mime_type: None,
            notes: None,
            net_cents: line_total_cents,
            vat_breakdown: vec![],
            gross_cents: line_total_cents,
            lines: vec![SupplierInvoiceLine {
                id: line_id,
                organization_id,
                supplier_invoice_id: invoice_id,
                label: "Plaques de plâtre".to_owned(),
                quantity: Decimal::from(10),
                unit: Some("u".to_owned()),
                unit_price_cents: line_total_cents / 10,
                line_total_cents,
                vat_rate_basis_points: Some(2000),
                position: 0,
                deleted_at: None,
                created_at: now,
                updated_at: now,
            }],
            deleted_at: None,
            created_at: now,
            updated_at: now,
        }
    }

    fn allocate_command(
        organization_id: OrganizationId,
        invoice_id: SupplierInvoiceId,
        line_id: SupplierInvoiceLineId,
        amount_cents: i32,
    ) -> AllocateSupplierInvoiceLineCommand {
        AllocateSupplierInvoiceLineCommand {
            organization_id,
            supplier_invoice_id: invoice_id,
            supplier_invoice_line_id: line_id,
            project_id: ProjectId(uuid::Uuid::new_v4()),
            amount_cents,
        }
    }

    #[tokio::test]
    async fn allocate_line_persists_within_bound_and_emits() {
        let organization_id = OrganizationId(uuid::Uuid::new_v4());
        let invoice_id = SupplierInvoiceId(uuid::Uuid::new_v4());
        let line_id = SupplierInvoiceLineId(uuid::Uuid::new_v4());
        let invoice = invoice_with_line(organization_id, invoice_id, line_id, 10_000);

        let mut repo = MockSupplierInvoiceRepository::new();
        repo.expect_find_by_id().with(always()).returning(move |_| {
            let invoice = invoice.clone();
            Box::pin(async move { Ok(Some(invoice)) })
        });

        let mut allocation_repo = MockSupplierInvoiceAllocationRepository::new();
        allocation_repo
            .expect_sum_allocated_for_line()
            .with(always())
            .returning(|_| Box::pin(async move { Ok(4_000) }));
        allocation_repo
            .expect_insert()
            .withf(|allocation| allocation.amount_cents == 6_000)
            .returning(|allocation| {
                let allocation = allocation.clone();
                Box::pin(async move { Ok(allocation) })
            });

        let emitter = RecordingEmitter::new();
        let mut service = SupplierInvoiceService::new(repo, &emitter);
        let command = allocate_command(organization_id, invoice_id, line_id, 6_000);

        let created = service
            .allocate_line(command, allocation_repo)
            .await
            .unwrap();

        assert_eq!(created.amount_cents, 6_000);
        assert_eq!(created.supplier_invoice_line_id, line_id);
        assert_eq!(
            emitter.names(),
            vec!["supplier_invoice_line_allocation.recorded"]
        );
    }

    #[tokio::test]
    async fn allocate_line_refuses_an_overflow() {
        let organization_id = OrganizationId(uuid::Uuid::new_v4());
        let invoice_id = SupplierInvoiceId(uuid::Uuid::new_v4());
        let line_id = SupplierInvoiceLineId(uuid::Uuid::new_v4());
        let invoice = invoice_with_line(organization_id, invoice_id, line_id, 10_000);

        let mut repo = MockSupplierInvoiceRepository::new();
        repo.expect_find_by_id().with(always()).returning(move |_| {
            let invoice = invoice.clone();
            Box::pin(async move { Ok(Some(invoice)) })
        });

        let mut allocation_repo = MockSupplierInvoiceAllocationRepository::new();
        allocation_repo
            .expect_sum_allocated_for_line()
            .with(always())
            .returning(|_| Box::pin(async move { Ok(9_000) }));
        // `insert` is deliberately given no expectation: the overflow must be
        // refused before the service ever reaches for it.

        let emitter = RecordingEmitter::new();
        let mut service = SupplierInvoiceService::new(repo, &emitter);
        let command = allocate_command(organization_id, invoice_id, line_id, 2_000);

        let result = service.allocate_line(command, allocation_repo).await;

        assert!(matches!(result, Err(CoreError::Conflict(_))));
        assert!(emitter.names().is_empty());
    }

    #[tokio::test]
    async fn allocate_line_refuses_a_zero_amount() {
        let organization_id = OrganizationId(uuid::Uuid::new_v4());
        let invoice_id = SupplierInvoiceId(uuid::Uuid::new_v4());
        let line_id = SupplierInvoiceLineId(uuid::Uuid::new_v4());

        // No expectations set on either mock: a zero amount is refused before
        // either repository is ever touched.
        let repo = MockSupplierInvoiceRepository::new();
        let allocation_repo = MockSupplierInvoiceAllocationRepository::new();

        let emitter = RecordingEmitter::new();
        let mut service = SupplierInvoiceService::new(repo, &emitter);
        let command = allocate_command(organization_id, invoice_id, line_id, 0);

        let result = service.allocate_line(command, allocation_repo).await;

        assert!(matches!(result, Err(CoreError::Conflict(_))));
    }

    #[tokio::test]
    async fn allocate_line_refuses_a_sign_mismatch_with_a_credit_line() {
        let organization_id = OrganizationId(uuid::Uuid::new_v4());
        let invoice_id = SupplierInvoiceId(uuid::Uuid::new_v4());
        let line_id = SupplierInvoiceLineId(uuid::Uuid::new_v4());
        // A credit/rebate line: a negative total.
        let invoice = invoice_with_line(organization_id, invoice_id, line_id, -5_000);

        let mut repo = MockSupplierInvoiceRepository::new();
        repo.expect_find_by_id().with(always()).returning(move |_| {
            let invoice = invoice.clone();
            Box::pin(async move { Ok(Some(invoice)) })
        });

        let allocation_repo = MockSupplierInvoiceAllocationRepository::new();

        let emitter = RecordingEmitter::new();
        let mut service = SupplierInvoiceService::new(repo, &emitter);
        // A positive allocation against a negative (credit) line.
        let command = allocate_command(organization_id, invoice_id, line_id, 1_000);

        let result = service.allocate_line(command, allocation_repo).await;

        assert!(matches!(result, Err(CoreError::Conflict(_))));
    }

    #[tokio::test]
    async fn allocate_line_refuses_a_line_the_invoice_does_not_carry() {
        let organization_id = OrganizationId(uuid::Uuid::new_v4());
        let invoice_id = SupplierInvoiceId(uuid::Uuid::new_v4());
        let line_id = SupplierInvoiceLineId(uuid::Uuid::new_v4());
        let invoice = invoice_with_line(organization_id, invoice_id, line_id, 10_000);

        let mut repo = MockSupplierInvoiceRepository::new();
        repo.expect_find_by_id().with(always()).returning(move |_| {
            let invoice = invoice.clone();
            Box::pin(async move { Ok(Some(invoice)) })
        });

        let allocation_repo = MockSupplierInvoiceAllocationRepository::new();

        let emitter = RecordingEmitter::new();
        let mut service = SupplierInvoiceService::new(repo, &emitter);
        let other_line_id = SupplierInvoiceLineId(uuid::Uuid::new_v4());
        let command = allocate_command(organization_id, invoice_id, other_line_id, 1_000);

        let result = service.allocate_line(command, allocation_repo).await;

        assert!(matches!(result, Err(CoreError::NotFound)));
    }

    #[tokio::test]
    async fn allocated_cost_for_project_sums_the_recorded_allocations() {
        let project_id = ProjectId(uuid::Uuid::new_v4());
        let organization_id = OrganizationId(uuid::Uuid::new_v4());
        let now = Utc::now();

        let mut allocation_repo = MockSupplierInvoiceAllocationRepository::new();
        allocation_repo
            .expect_list_by_project()
            .with(always())
            .returning(move |project_id| {
                let rows = vec![
                    SupplierInvoiceLineAllocation {
                        id: SupplierInvoiceLineAllocationId(uuid::Uuid::new_v4()),
                        organization_id,
                        supplier_invoice_line_id: SupplierInvoiceLineId(uuid::Uuid::new_v4()),
                        project_id,
                        amount_cents: 6_000,
                        created_at: now,
                        updated_at: now,
                    },
                    SupplierInvoiceLineAllocation {
                        id: SupplierInvoiceLineAllocationId(uuid::Uuid::new_v4()),
                        organization_id,
                        supplier_invoice_line_id: SupplierInvoiceLineId(uuid::Uuid::new_v4()),
                        project_id,
                        amount_cents: 4_000,
                        created_at: now,
                        updated_at: now,
                    },
                ];
                Box::pin(async move { Ok(rows) })
            });

        let repo = MockSupplierInvoiceRepository::new();
        let emitter = RecordingEmitter::new();
        let mut service = SupplierInvoiceService::new(repo, &emitter);

        let total = service
            .allocated_cost_for_project(project_id, allocation_repo)
            .await
            .unwrap();

        assert_eq!(total, 10_000);
    }

    #[tokio::test]
    async fn update_notes_persists_regardless_of_status() {
        let mut repo = MockSupplierInvoiceRepository::new();
        repo.expect_find_by_id()
            .with(always())
            .returning(|id| Box::pin(async move { Ok(Some(confirmed_fixture(id))) }));
        repo.expect_update_review().returning(|review| {
            let invoice = review.invoice().clone();
            Box::pin(async move { Ok(invoice) })
        });

        let emitter = RecordingEmitter::new();
        let mut service = SupplierInvoiceService::new(repo, &emitter);
        let updated = service
            .update_notes(UpdateSupplierInvoiceNotesCommand {
                id: SupplierInvoiceId(uuid::Uuid::new_v4()),
                notes: Some("À vérifier avec le fournisseur".to_owned()),
            })
            .await
            .unwrap();

        // The fixture is `Confirmed` — a status update_notes must not touch.
        assert_eq!(updated.status, SupplierInvoiceStatus::Confirmed);
    }

    #[tokio::test]
    async fn replace_line_allocations_keeps_unchanged_inserts_new_and_drops_missing() {
        let organization_id = OrganizationId(uuid::Uuid::new_v4());
        let invoice_id = SupplierInvoiceId(uuid::Uuid::new_v4());
        let line_id = SupplierInvoiceLineId(uuid::Uuid::new_v4());
        let invoice = invoice_with_line(organization_id, invoice_id, line_id, 10_000);

        let mut repo = MockSupplierInvoiceRepository::new();
        repo.expect_find_by_id().with(always()).returning(move |_| {
            let invoice = invoice.clone();
            Box::pin(async move { Ok(Some(invoice)) })
        });

        let kept_project_id = ProjectId(uuid::Uuid::new_v4());
        let dropped_project_id = ProjectId(uuid::Uuid::new_v4());
        let new_project_id = ProjectId(uuid::Uuid::new_v4());
        let now = Utc::now();
        let kept_id = SupplierInvoiceLineAllocationId(uuid::Uuid::new_v4());
        let dropped_id = SupplierInvoiceLineAllocationId(uuid::Uuid::new_v4());

        let mut allocation_repo = MockSupplierInvoiceAllocationRepository::new();
        allocation_repo
            .expect_list_by_line()
            .with(always())
            .returning(move |_| {
                let rows = vec![
                    SupplierInvoiceLineAllocation {
                        id: kept_id,
                        organization_id,
                        supplier_invoice_line_id: line_id,
                        project_id: kept_project_id,
                        amount_cents: 4_000,
                        created_at: now,
                        updated_at: now,
                    },
                    SupplierInvoiceLineAllocation {
                        id: dropped_id,
                        organization_id,
                        supplier_invoice_line_id: line_id,
                        project_id: dropped_project_id,
                        amount_cents: 2_000,
                        created_at: now,
                        updated_at: now,
                    },
                ];
                Box::pin(async move { Ok(rows) })
            });
        allocation_repo
            .expect_delete()
            .withf(move |id| *id == dropped_id)
            .returning(|_| Box::pin(async move { Ok(()) }));
        allocation_repo
            .expect_insert()
            .withf(move |allocation| {
                allocation.project_id == new_project_id && allocation.amount_cents == 3_000
            })
            .returning(|allocation| {
                let allocation = allocation.clone();
                Box::pin(async move { Ok(allocation) })
            });

        let emitter = RecordingEmitter::new();
        let mut service = SupplierInvoiceService::new(repo, &emitter);
        let result = service
            .replace_line_allocations(
                ReplaceSupplierInvoiceLineAllocationsCommand {
                    organization_id,
                    supplier_invoice_id: invoice_id,
                    supplier_invoice_line_id: line_id,
                    allocations: vec![
                        LineAllocationShare {
                            project_id: kept_project_id,
                            amount_cents: 4_000,
                        },
                        LineAllocationShare {
                            project_id: new_project_id,
                            amount_cents: 3_000,
                        },
                    ],
                },
                allocation_repo,
            )
            .await
            .unwrap();

        assert_eq!(result.len(), 2);
        assert!(result.iter().any(|a| a.id == kept_id));
        assert!(result.iter().any(|a| a.project_id == new_project_id));
        assert_eq!(
            emitter.names(),
            vec!["supplier_invoice_line_allocation.recorded"]
        );
    }

    #[tokio::test]
    async fn replace_line_allocations_refuses_a_set_that_overflows_the_line() {
        let organization_id = OrganizationId(uuid::Uuid::new_v4());
        let invoice_id = SupplierInvoiceId(uuid::Uuid::new_v4());
        let line_id = SupplierInvoiceLineId(uuid::Uuid::new_v4());
        let invoice = invoice_with_line(organization_id, invoice_id, line_id, 10_000);

        let mut repo = MockSupplierInvoiceRepository::new();
        repo.expect_find_by_id().with(always()).returning(move |_| {
            let invoice = invoice.clone();
            Box::pin(async move { Ok(Some(invoice)) })
        });

        // No expectations on the allocation mock: an overflowing set must be
        // refused before it is ever read or written.
        let allocation_repo = MockSupplierInvoiceAllocationRepository::new();

        let emitter = RecordingEmitter::new();
        let mut service = SupplierInvoiceService::new(repo, &emitter);
        let result = service
            .replace_line_allocations(
                ReplaceSupplierInvoiceLineAllocationsCommand {
                    organization_id,
                    supplier_invoice_id: invoice_id,
                    supplier_invoice_line_id: line_id,
                    allocations: vec![LineAllocationShare {
                        project_id: ProjectId(uuid::Uuid::new_v4()),
                        amount_cents: 10_001,
                    }],
                },
                allocation_repo,
            )
            .await;

        assert!(matches!(result, Err(CoreError::Conflict(_))));
    }
}
