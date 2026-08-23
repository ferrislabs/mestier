use std::collections::BTreeMap;

use chrono::{Datelike, Utc};
use common::{CoreError, generate_uuid_v7};
use events::EventEmitter;
use rust_decimal::{Decimal, prelude::ToPrimitive};
use serde_json::{Value, json};

use crate::{
    CustomerContextId, CustomerId, DraftInvoice, Invoice, InvoiceId, InvoiceKind, InvoiceLine,
    InvoiceLineId, InvoiceStatus, InvoiceVatBreakdownLine, LegalIdentity, Organization,
    OrganizationId, Project, ProjectId,
    domain::{
        invoice::{
            commands::{
                CancelInvoiceCommand, CreateInvoiceCommand, InvoiceLineCommand,
                IssueDepositCommand, IssueFinalInvoiceCommand, IssueInvoiceCommand,
                UpdateInvoiceCommand,
            },
            events::{InvoiceCreated, InvoiceDeleted, InvoiceTransitioned, InvoiceUpdated},
            ports::InvoiceRepository,
        },
        organization::{legal_identity::VatStatus, ports::OrganizationRepository},
        project::{ports::ProjectRepository, service::remaining_to_bill_cents},
        quote::ports::QuoteRepository,
    },
};

pub struct InvoiceService<R, E>
where
    R: InvoiceRepository,
    E: EventEmitter,
{
    repo: R,
    emitter: E,
}

impl<R, E> InvoiceService<R, E>
where
    R: InvoiceRepository,
    E: EventEmitter,
{
    pub fn new(repo: R, emitter: E) -> Self {
        Self { repo, emitter }
    }

    pub async fn create_invoice<O>(
        &mut self,
        command: CreateInvoiceCommand,
        mut organization_repository: O,
    ) -> Result<Invoice, CoreError>
    where
        O: OrganizationRepository,
    {
        let now = Utc::now();
        let invoice_id = InvoiceId(generate_uuid_v7());
        let organization =
            resolve_organization(&mut organization_repository, command.organization_id).await?;
        let lines = build_invoice_lines(command.organization_id, invoice_id, command.lines, now)?;
        let totals = calculate_totals(&lines, organization.vat_status.as_ref())?;

        let draft = DraftInvoice::try_from_invoice(Invoice {
            id: invoice_id,
            organization_id: command.organization_id,
            number: None,
            kind: command.kind,
            project_id: command.project_id,
            customer_id: command.customer_id,
            customer_context_id: command.customer_context_id,
            status: InvoiceStatus::Draft,
            issued_at: None,
            due_at: command.due_at,
            notes: command.notes,
            operation_nature: command.operation_nature,
            delivery_address: command.delivery_address,
            net_cents: totals.net_cents,
            vat_breakdown: totals.vat_breakdown,
            gross_cents: totals.gross_cents,
            issuer_identity: None,
            lines,
            deleted_at: None,
            created_at: now,
            updated_at: now,
        })
        .expect("just constructed with status Draft");

        let created = self.repo.insert_draft(&draft).await?;

        self.emitter.emit(
            created.organization_id,
            &InvoiceCreated {
                invoice: created.clone(),
            },
        )?;

        Ok(created)
    }

    pub async fn get_invoice(&mut self, id: InvoiceId) -> Result<Invoice, CoreError> {
        self.repo.find_by_id(id).await?.ok_or(CoreError::NotFound)
    }

    pub async fn list_invoices(
        &mut self,
        organization_id: OrganizationId,
        limit: u64,
        offset: u64,
    ) -> Result<(Vec<Invoice>, u64), CoreError> {
        self.repo
            .list_by_organization(organization_id, limit, offset)
            .await
    }

    pub async fn list_invoices_by_project(
        &mut self,
        project_id: crate::ProjectId,
    ) -> Result<Vec<Invoice>, CoreError> {
        self.repo.list_by_project(project_id).await
    }

    /// Refused, by [`DraftInvoice::try_from_invoice`], on anything but a
    /// draft: the compile-time guarantee is that `update_draft` cannot be
    /// called with anything else.
    pub async fn update_invoice<O>(
        &mut self,
        command: UpdateInvoiceCommand,
        mut organization_repository: O,
    ) -> Result<Invoice, CoreError>
    where
        O: OrganizationRepository,
    {
        let existing = self.get_invoice(command.id).await?;
        let now = Utc::now();
        let organization =
            resolve_organization(&mut organization_repository, existing.organization_id).await?;
        let lines = build_invoice_lines(existing.organization_id, existing.id, command.lines, now)?;
        let totals = calculate_totals(&lines, organization.vat_status.as_ref())?;

        let mut draft = DraftInvoice::try_from_invoice(existing.clone())?;
        draft.set_project(command.project_id);
        draft.set_customer(command.customer_id, command.customer_context_id);
        draft.set_due_at(command.due_at);
        draft.set_notes(command.notes);
        draft.set_operation_nature(command.operation_nature);
        draft.set_delivery_address(command.delivery_address);
        draft.set_lines_and_totals(
            lines,
            totals.net_cents,
            totals.vat_breakdown,
            totals.gross_cents,
        );
        draft.touch(now);

        let updated = self.repo.update_draft(&draft).await?;

        let (changed_fields, previous) = content_diff(&existing, &updated);
        if !changed_fields.is_empty() {
            self.emitter.emit(
                updated.organization_id,
                &InvoiceUpdated {
                    invoice: updated.clone(),
                    changed_fields,
                    previous,
                },
            )?;
        }

        Ok(updated)
    }

    /// Draft or issued, never a document already `Cancelled`, `Paid` or
    /// `PartiallyPaid` — those transitions belong to issuing (#317) and to
    /// payment recording (#318, #320), never to a bare status write here.
    pub async fn cancel_invoice(
        &mut self,
        command: CancelInvoiceCommand,
    ) -> Result<Invoice, CoreError> {
        let existing = self.get_invoice(command.id).await?;
        if matches!(
            existing.status,
            InvoiceStatus::Cancelled | InvoiceStatus::Paid | InvoiceStatus::PartiallyPaid
        ) {
            return Err(CoreError::Conflict(format!(
                "invoice {} is {} and cannot be cancelled",
                existing.id, existing.status
            )));
        }

        let updated = self
            .repo
            .update_status(command.id, InvoiceStatus::Cancelled, Utc::now())
            .await?;

        self.emit_transition(existing.status, &updated)?;

        Ok(updated)
    }

    /// Refused unless the invoice is still a draft: an issued invoice is a
    /// legal document, corrected with a credit note (#318), never deleted.
    pub async fn soft_delete_invoice(&mut self, id: InvoiceId) -> Result<(), CoreError> {
        let existing = self.get_invoice(id).await?;
        if existing.status != InvoiceStatus::Draft {
            return Err(CoreError::Conflict(format!(
                "invoice {} is {} and cannot be deleted; only a draft can be",
                existing.id, existing.status
            )));
        }

        self.repo.soft_delete(id, Utc::now()).await?;

        self.emitter
            .emit(existing.organization_id, &InvoiceDeleted { invoice_id: id })
    }

    /// The generic issuing transition, usable on any draft regardless of
    /// `kind` — a `Standard` draft from #316's `create_invoice`, or a
    /// `Deposit`/`Final` draft this service builds itself. Refused,
    /// through [`DraftInvoice::try_from_invoice`], on anything but a draft:
    /// no parallel status check, that compile-time-enforced path is reused
    /// as-is.
    pub async fn issue_invoice<O, P, Q>(
        &mut self,
        command: IssueInvoiceCommand,
        organization_repository: O,
        project_repository: P,
        quote_repository: Q,
    ) -> Result<Invoice, CoreError>
    where
        O: OrganizationRepository,
        P: ProjectRepository,
        Q: QuoteRepository,
    {
        let existing = self.get_invoice(command.id).await?;
        let draft = DraftInvoice::try_from_invoice(existing)?;

        self.issue_now(
            draft,
            command.allow_exceeding_total,
            organization_repository,
            project_repository,
            quote_repository,
        )
        .await
    }

    /// Builds a deposit line for `percentage_bp` of the project's quoted
    /// net total and issues it in one transaction. Needs a quote to compute
    /// a percentage against — specific to this act, not a general rule:
    /// `create_invoice`/`issue_invoice` still work with no project, or a
    /// project with no quote.
    pub async fn issue_deposit<O, P, Q>(
        &mut self,
        command: IssueDepositCommand,
        organization_repository: O,
        mut project_repository: P,
        mut quote_repository: Q,
    ) -> Result<Invoice, CoreError>
    where
        O: OrganizationRepository,
        P: ProjectRepository,
        Q: QuoteRepository,
    {
        validate_percentage_bp(command.percentage_bp)?;

        let project = project_repository
            .find_by_id(command.project_id)
            .await?
            .ok_or(CoreError::NotFound)?;
        let quote_id = project.quote_id.ok_or_else(|| {
            CoreError::Conflict(format!(
                "project {} has no quote; a deposit needs a total to compute a percentage against",
                project.id
            ))
        })?;
        let (customer_id, customer_context_id) = required_customer_fields(&project)?;
        let quote = quote_repository
            .find_by_id(quote_id)
            .await?
            .ok_or(CoreError::NotFound)?;

        let amount_cents = deposit_amount_cents(command.percentage_bp, quote.net_cents)?;

        // Fail fast, before any row is created: a deposit's amount is a
        // percentage of the whole quote, not of what remains, so it can
        // land past the quote's total on its own even though a final
        // invoice's amount (computed from `remaining_to_bill_cents`
        // directly) structurally cannot.
        self.refuse_if_exceeding_total(
            project.id,
            quote.net_cents,
            amount_cents,
            command.allow_exceeding_total,
        )
        .await?;

        let now = Utc::now();
        let invoice_id = InvoiceId(generate_uuid_v7());
        let label = format!("Acompte ({})", format_percentage_bp(command.percentage_bp));
        let draft = build_single_line_draft(
            invoice_id,
            project.organization_id,
            InvoiceKind::Deposit,
            Some(project.id),
            customer_id,
            customer_context_id,
            command.due_at,
            command.notes,
            label,
            amount_cents,
            now,
        );

        let inserted = self.repo.insert_draft(&draft).await?;
        self.emitter.emit(
            inserted.organization_id,
            &InvoiceCreated {
                invoice: inserted.clone(),
            },
        )?;
        let inserted_draft =
            DraftInvoice::try_from_invoice(inserted).expect("insert_draft returns a draft");

        self.issue_now(
            inserted_draft,
            command.allow_exceeding_total,
            organization_repository,
            project_repository,
            quote_repository,
        )
        .await
    }

    /// Builds a line for exactly what remains to bill on the project's
    /// quote and issues it in one transaction. Same quote requirement, and
    /// the same reason, as [`Self::issue_deposit`].
    pub async fn issue_final_invoice<O, P, Q>(
        &mut self,
        command: IssueFinalInvoiceCommand,
        organization_repository: O,
        mut project_repository: P,
        mut quote_repository: Q,
    ) -> Result<Invoice, CoreError>
    where
        O: OrganizationRepository,
        P: ProjectRepository,
        Q: QuoteRepository,
    {
        let project = project_repository
            .find_by_id(command.project_id)
            .await?
            .ok_or(CoreError::NotFound)?;
        let quote_id = project.quote_id.ok_or_else(|| {
            CoreError::Conflict(format!(
                "project {} has no quote; a final invoice needs a total to compute the remainder against",
                project.id
            ))
        })?;
        let (customer_id, customer_context_id) = required_customer_fields(&project)?;
        let quote = quote_repository
            .find_by_id(quote_id)
            .await?
            .ok_or(CoreError::NotFound)?;

        let already_issued = self.sum_already_issued_cents(project.id).await?;
        let remaining = remaining_to_bill_cents(quote.net_cents, already_issued)?;
        // A final invoice's amount is, by construction, exactly what
        // remains: already_issued + remaining always equals the quoted
        // total, so this can never itself push the project past it. The
        // one case worth a named refusal is nothing (or less than nothing)
        // left to bill — a prior deposit already reached or exceeded the
        // quote, which `allow_exceeding_total` on an earlier act, not this
        // one, is what let happen.
        if remaining <= 0 {
            return Err(CoreError::Conflict(format!(
                "project {} has nothing left to bill: already issued {already_issued} cents \
                 against a quoted total of {} cents",
                project.id, quote.net_cents
            )));
        }
        let amount_cents = i32::try_from(remaining).map_err(|_| {
            CoreError::Conflict("remaining amount to bill is outside supported bounds".to_owned())
        })?;

        let now = Utc::now();
        let invoice_id = InvoiceId(generate_uuid_v7());
        let draft = build_single_line_draft(
            invoice_id,
            project.organization_id,
            InvoiceKind::Final,
            Some(project.id),
            customer_id,
            customer_context_id,
            command.due_at,
            command.notes,
            "Solde".to_owned(),
            amount_cents,
            now,
        );

        let inserted = self.repo.insert_draft(&draft).await?;
        self.emitter.emit(
            inserted.organization_id,
            &InvoiceCreated {
                invoice: inserted.clone(),
            },
        )?;
        let inserted_draft =
            DraftInvoice::try_from_invoice(inserted).expect("insert_draft returns a draft");

        self.issue_now(
            inserted_draft,
            command.allow_exceeding_total,
            organization_repository,
            project_repository,
            quote_repository,
        )
        .await
    }

    /// The shared primitive behind [`Self::issue_invoice`],
    /// [`Self::issue_deposit`] and [`Self::issue_final_invoice`]: resolves
    /// the issuer's identity (refused by type, not by check — the
    /// `LegalIdentity` either builds or it doesn't), refuses an
    /// over-issuance the caller has not explicitly allowed, allocates the
    /// number and persists the transition, all inside the one transaction
    /// the enclosing `#[transactional]` use case already opened. Numbering
    /// happening here, rather than one call up, is what makes a
    /// draft-build-then-issue sequence for deposit/final allocate its
    /// number in the same transaction as the draft it belongs to.
    async fn issue_now<O, P, Q>(
        &mut self,
        draft: DraftInvoice,
        allow_exceeding_total: bool,
        mut organization_repository: O,
        mut project_repository: P,
        mut quote_repository: Q,
    ) -> Result<Invoice, CoreError>
    where
        O: OrganizationRepository,
        P: ProjectRepository,
        Q: QuoteRepository,
    {
        let invoice = draft.into_invoice();

        let organization =
            resolve_organization(&mut organization_repository, invoice.organization_id).await?;
        let legal_identity =
            LegalIdentity::try_from_organization(&organization).map_err(|missing_fields| {
                CoreError::Conflict(format!(
                    "cannot issue invoice {}: the organization's legal identity is missing {}",
                    invoice.id,
                    missing_fields.join(", ")
                ))
            })?;

        self.refuse_if_project_exceeds_total(
            &invoice,
            allow_exceeding_total,
            &mut project_repository,
            &mut quote_repository,
        )
        .await?;

        let year = Utc::now().year();
        let number = self
            .repo
            .allocate_number(
                invoice.organization_id,
                &organization.invoice_number_prefix,
                year,
            )
            .await?;
        let now = Utc::now();

        let issued = self
            .repo
            .issue(invoice.id, number, now, &legal_identity, now)
            .await?;

        self.emit_transition(InvoiceStatus::Draft, &issued)?;

        Ok(issued)
    }

    /// The check [`IssueInvoiceCommand::allow_exceeding_total`] guards: does
    /// this invoice, added to every other non-draft non-cancelled invoice
    /// already on its project, push the total past the project's quote? A
    /// no-op when the invoice carries no project, or the project no quote —
    /// there is nothing to compare against either way.
    async fn refuse_if_project_exceeds_total(
        &mut self,
        invoice: &Invoice,
        allow_exceeding_total: bool,
        project_repository: &mut impl ProjectRepository,
        quote_repository: &mut impl QuoteRepository,
    ) -> Result<(), CoreError> {
        let Some(project_id) = invoice.project_id else {
            return Ok(());
        };
        let project = project_repository
            .find_by_id(project_id)
            .await?
            .ok_or(CoreError::NotFound)?;
        let Some(quote_id) = project.quote_id else {
            return Ok(());
        };
        let quote = quote_repository
            .find_by_id(quote_id)
            .await?
            .ok_or(CoreError::NotFound)?;

        // The invoice being issued right now is either not persisted yet
        // (issue_deposit/issue_final_invoice) or still `Draft` (any other
        // caller of issue_invoice, since this only runs on a draft) — either
        // way `sum_already_issued_cents`, called inside
        // `refuse_if_exceeding_total`, excludes it on its own, no filter by
        // id needed here.
        self.refuse_if_exceeding_total(
            project_id,
            quote.net_cents,
            invoice.net_cents,
            allow_exceeding_total,
        )
        .await
    }

    /// Refuses when `amount_cents`, added to what the project has already
    /// been invoiced for, would land past `quoted_cents` — unless the
    /// caller explicitly allows it. The one place both the generic issuing
    /// check and `issue_deposit`'s own pre-check land, so neither re-derives
    /// the subtraction independently.
    async fn refuse_if_exceeding_total(
        &mut self,
        project_id: ProjectId,
        quoted_cents: i32,
        amount_cents: i32,
        allow_exceeding_total: bool,
    ) -> Result<(), CoreError> {
        let already_issued = self.sum_already_issued_cents(project_id).await?;
        let remaining = remaining_to_bill_cents(quoted_cents, already_issued)?;

        if i64::from(amount_cents) > remaining && !allow_exceeding_total {
            return Err(CoreError::Conflict(format!(
                "issuing {amount_cents} cents against project {project_id} would exceed its \
                 quoted total of {quoted_cents} cents: {already_issued} cents already issued, \
                 {remaining} cents remaining"
            )));
        }

        Ok(())
    }

    /// Every non-draft, non-cancelled invoice already on the project,
    /// summed. Deposits and finals already issued count in full — credit
    /// notes don't exist until #318, so nothing is netted out yet; #318
    /// will need to revisit this sum to subtract issued credit notes, or a
    /// partial refund will not reduce what counts as "already invoiced".
    async fn sum_already_issued_cents(&mut self, project_id: ProjectId) -> Result<i64, CoreError> {
        let invoices = self.repo.list_by_project(project_id).await?;

        Ok(invoices
            .into_iter()
            .filter(|invoice| {
                !matches!(
                    invoice.status,
                    InvoiceStatus::Draft | InvoiceStatus::Cancelled
                )
            })
            .map(|invoice| i64::from(invoice.net_cents))
            .sum())
    }

    fn emit_transition(&self, from: InvoiceStatus, invoice: &Invoice) -> Result<(), CoreError> {
        if from == invoice.status || InvoiceTransitioned::event_name(invoice.status).is_none() {
            return Ok(());
        }

        self.emitter.emit(
            invoice.organization_id,
            &InvoiceTransitioned {
                invoice: invoice.clone(),
                from,
            },
        )
    }
}

async fn resolve_organization(
    organization_repository: &mut impl OrganizationRepository,
    organization_id: OrganizationId,
) -> Result<Organization, CoreError> {
    organization_repository
        .find_by_id(organization_id)
        .await?
        .ok_or(CoreError::NotFound)
}

/// Which content fields moved, and what they held before. `status` is
/// absent by construction, same rule as `quote::service::content_diff`.
fn content_diff(existing: &Invoice, updated: &Invoice) -> (Vec<&'static str>, Value) {
    let mut changed = Vec::new();
    let mut previous = serde_json::Map::new();

    if existing.project_id != updated.project_id {
        changed.push("project_id");
        previous.insert(
            "project_id".to_owned(),
            json!(existing.project_id.map(|id| id.0)),
        );
    }
    if existing.customer_id != updated.customer_id {
        changed.push("customer_id");
        previous.insert("customer_id".to_owned(), json!(existing.customer_id.0));
    }
    if existing.customer_context_id != updated.customer_context_id {
        changed.push("customer_context_id");
        previous.insert(
            "customer_context_id".to_owned(),
            json!(existing.customer_context_id.0),
        );
    }
    if existing.due_at != updated.due_at {
        changed.push("due_at");
        previous.insert("due_at".to_owned(), json!(existing.due_at));
    }
    if existing.notes != updated.notes {
        changed.push("notes");
        previous.insert("notes".to_owned(), json!(existing.notes));
    }
    if existing.operation_nature != updated.operation_nature {
        changed.push("operation_nature");
        previous.insert(
            "operation_nature".to_owned(),
            json!(existing.operation_nature.map(|n| n.as_str())),
        );
    }
    if existing.delivery_address != updated.delivery_address {
        changed.push("delivery_address");
        previous.insert(
            "delivery_address".to_owned(),
            json!(existing.delivery_address.is_some()),
        );
    }
    if line_projection(existing) != line_projection(updated) {
        changed.push("lines");
        previous.insert("lines".to_owned(), line_projection(existing));
        previous.insert("net_cents".to_owned(), json!(existing.net_cents));
        previous.insert("gross_cents".to_owned(), json!(existing.gross_cents));
    }

    (changed, Value::Object(previous))
}

fn line_projection(invoice: &Invoice) -> Value {
    json!(
        invoice
            .lines
            .iter()
            .map(|line| json!({
                "label": line.label,
                "quantity": line.quantity.to_string(),
                "unit_price_cents": line.unit_price_cents,
                "vat_rate_basis_points": line.vat_rate_basis_points,
                "position": line.position,
            }))
            .collect::<Vec<_>>()
    )
}

fn build_invoice_lines(
    organization_id: OrganizationId,
    invoice_id: InvoiceId,
    commands: Vec<InvoiceLineCommand>,
    now: chrono::DateTime<Utc>,
) -> Result<Vec<InvoiceLine>, CoreError> {
    if commands.is_empty() {
        return Err(CoreError::Conflict(
            "an invoice must have at least one line".to_owned(),
        ));
    }

    commands
        .into_iter()
        .enumerate()
        .map(|(position, command)| {
            build_invoice_line(organization_id, invoice_id, command, position as i32, now)
        })
        .collect()
}

fn build_invoice_line(
    organization_id: OrganizationId,
    invoice_id: InvoiceId,
    command: InvoiceLineCommand,
    position: i32,
    now: chrono::DateTime<Utc>,
) -> Result<InvoiceLine, CoreError> {
    validate_line(&command)?;

    Ok(InvoiceLine {
        id: InvoiceLineId(generate_uuid_v7()),
        organization_id,
        invoice_id,
        label: command.label.trim().to_owned(),
        quantity: command.quantity,
        unit_price_cents: command.unit_price_cents,
        vat_rate_basis_points: command.vat_rate_basis_points,
        position,
        deleted_at: None,
        created_at: now,
        updated_at: now,
    })
}

fn validate_line(command: &InvoiceLineCommand) -> Result<(), CoreError> {
    if command.label.trim().is_empty() {
        return Err(CoreError::Conflict(
            "invoice line label cannot be empty".to_owned(),
        ));
    }

    if command.quantity <= Decimal::ZERO {
        return Err(CoreError::Conflict(
            "invoice line quantity must be positive".to_owned(),
        ));
    }

    if command.unit_price_cents < 0 {
        return Err(CoreError::Conflict(
            "invoice line unit price cannot be negative".to_owned(),
        ));
    }

    if command
        .vat_rate_basis_points
        .is_some_and(|rate_bp| !(0..=10_000).contains(&rate_bp))
    {
        return Err(CoreError::Conflict(
            "invoice line vat rate must be between 0 and 10000 basis points".to_owned(),
        ));
    }

    Ok(())
}

/// The two customer fields a project must carry before `issue_deposit` or
/// `issue_final_invoice` can build an invoice from it. `customer_id` absent
/// means an internal project — nothing to invoice (see
/// `Project::is_internal`). `customer_context_id` absent is refused too,
/// even though the field stays optional on `Project` itself (a customer
/// without a pinned site is a legitimate project state):
/// `Invoice::customer_context_id` is not optional, so a context has to be
/// pinned before either act can proceed.
fn required_customer_fields(
    project: &Project,
) -> Result<(CustomerId, CustomerContextId), CoreError> {
    let customer_id = project.customer_id.ok_or_else(|| {
        CoreError::Conflict(format!(
            "project {} has no customer; nothing to invoice",
            project.id
        ))
    })?;
    let customer_context_id = project.customer_context_id.ok_or_else(|| {
        CoreError::Conflict(format!(
            "project {} has a customer but no pinned customer context; an invoice needs one",
            project.id
        ))
    })?;

    Ok((customer_id, customer_context_id))
}

fn validate_percentage_bp(percentage_bp: i32) -> Result<(), CoreError> {
    if !(1..=10_000).contains(&percentage_bp) {
        return Err(CoreError::Conflict(
            "deposit percentage must be between 1 and 10000 basis points".to_owned(),
        ));
    }

    Ok(())
}

/// `percentage_bp` of `quoted_cents`, rounded half to even — reuses
/// `div_round_half_even`, the one function in this module that already
/// knows how to round a basis-points fraction of a cents amount, rather
/// than re-deriving the rounding rule for a deposit.
fn deposit_amount_cents(percentage_bp: i32, quoted_cents: i32) -> Result<i32, CoreError> {
    let amount = div_round_half_even(i64::from(percentage_bp) * i64::from(quoted_cents), 10_000);

    i32::try_from(amount)
        .map_err(|_| CoreError::Conflict("deposit amount is outside supported bounds".to_owned()))
}

/// Basis points to a percentage string for a deposit's label: 3000 ->
/// "30%", 3333 -> "33.33%". Trailing zero hundredths are dropped so a round
/// deposit reads "30%", not "30.00%".
fn format_percentage_bp(percentage_bp: i32) -> String {
    let whole = percentage_bp / 100;
    let hundredths = percentage_bp % 100;

    if hundredths == 0 {
        format!("{whole}%")
    } else {
        format!("{whole}.{hundredths:02}%")
    }
}

/// Builds the single-line draft `issue_deposit` and `issue_final_invoice`
/// both persist before handing off to `issue_now`. `vat_rate_basis_points`
/// is deliberately `None`: a deposit or a final amount is a portion of an
/// already-VAT-computed quote total, not a fresh basis to re-derive a VAT
/// breakdown from. `calculate_totals`'s equivalent (not called here, since
/// there is only ever one line and its totals are already known) would
/// price a `None`-rate line at the implicit 0bp rate, so `net_cents` and
/// `gross_cents` are set equal here regardless of the organization's VAT
/// status — a known simplification, not a silent one: a deposit/final line
/// showing its own VAT breakdown is a real gap, left for a follow-up.
#[allow(clippy::too_many_arguments)]
fn build_single_line_draft(
    invoice_id: InvoiceId,
    organization_id: OrganizationId,
    kind: InvoiceKind,
    project_id: Option<ProjectId>,
    customer_id: CustomerId,
    customer_context_id: CustomerContextId,
    due_at: Option<chrono::DateTime<Utc>>,
    notes: Option<String>,
    label: String,
    amount_cents: i32,
    now: chrono::DateTime<Utc>,
) -> DraftInvoice {
    let line = InvoiceLine {
        id: InvoiceLineId(generate_uuid_v7()),
        organization_id,
        invoice_id,
        label,
        quantity: Decimal::ONE,
        unit_price_cents: amount_cents,
        vat_rate_basis_points: None,
        position: 0,
        deleted_at: None,
        created_at: now,
        updated_at: now,
    };

    DraftInvoice::try_from_invoice(Invoice {
        id: invoice_id,
        organization_id,
        number: None,
        kind,
        project_id,
        customer_id,
        customer_context_id,
        status: InvoiceStatus::Draft,
        issued_at: None,
        due_at,
        notes,
        operation_nature: None,
        delivery_address: None,
        net_cents: amount_cents,
        vat_breakdown: Vec::new(),
        gross_cents: amount_cents,
        issuer_identity: None,
        lines: vec![line],
        deleted_at: None,
        created_at: now,
        updated_at: now,
    })
    .expect("just constructed with status Draft")
}

pub(crate) struct InvoiceTotals {
    pub net_cents: i32,
    pub vat_breakdown: Vec<InvoiceVatBreakdownLine>,
    pub gross_cents: i32,
}

fn line_net_cents(quantity: Decimal, unit_price_cents: i32) -> Result<i64, CoreError> {
    let line_total = (quantity * Decimal::from(unit_price_cents)).round_dp(0);

    line_total.to_i64().ok_or_else(|| {
        CoreError::Conflict("invoice line total is outside supported bounds".to_owned())
    })
}

fn calculate_total_cents(lines: &[InvoiceLine]) -> Result<i32, CoreError> {
    let total = lines.iter().try_fold(0_i64, |sum, line| {
        let line_total = line_net_cents(line.quantity, line.unit_price_cents)?;

        sum.checked_add(line_total).ok_or_else(|| {
            CoreError::Conflict("invoice total is outside supported bounds".to_owned())
        })
    })?;

    i32::try_from(total)
        .map_err(|_| CoreError::Conflict("invoice total is outside supported bounds".to_owned()))
}

/// Net, VAT broken down per rate, and gross — same rule as
/// `quote::service::calculate_totals`: rounded per line then summed per
/// rate, not summed then rounded, and an organization not (yet) subject to
/// VAT produces `gross_cents == net_cents` with an empty breakdown.
pub(crate) fn calculate_totals(
    lines: &[InvoiceLine],
    vat_status: Option<&VatStatus>,
) -> Result<InvoiceTotals, CoreError> {
    let net_cents = calculate_total_cents(lines)?;

    if !matches!(vat_status, Some(VatStatus::Subject { .. })) {
        return Ok(InvoiceTotals {
            net_cents,
            vat_breakdown: Vec::new(),
            gross_cents: net_cents,
        });
    }

    let mut vat_by_rate: BTreeMap<i32, i64> = BTreeMap::new();
    let mut vat_total: i64 = 0;

    for line in lines {
        let rate_bp = line.vat_rate_basis_points.unwrap_or(0);
        let line_net = line_net_cents(line.quantity, line.unit_price_cents)?;
        let line_vat = div_round_half_even(line_net * i64::from(rate_bp), 10_000);

        *vat_by_rate.entry(rate_bp).or_insert(0) += line_vat;
        vat_total = vat_total.checked_add(line_vat).ok_or_else(|| {
            CoreError::Conflict("invoice VAT total is outside supported bounds".to_owned())
        })?;
    }

    let vat_breakdown = vat_by_rate
        .into_iter()
        .map(|(rate_bp, vat_cents)| {
            i32::try_from(vat_cents)
                .map(|vat_cents| InvoiceVatBreakdownLine { rate_bp, vat_cents })
                .map_err(|_| {
                    CoreError::Conflict("invoice VAT total is outside supported bounds".to_owned())
                })
        })
        .collect::<Result<Vec<_>, CoreError>>()?;

    let gross_cents = i64::from(net_cents)
        .checked_add(vat_total)
        .and_then(|total| i32::try_from(total).ok())
        .ok_or_else(|| {
            CoreError::Conflict("invoice total is outside supported bounds".to_owned())
        })?;

    Ok(InvoiceTotals {
        net_cents,
        vat_breakdown,
        gross_cents,
    })
}

/// Divides, rounding a half to the even neighbour. Duplicated from
/// `quote::service::div_round_half_even` — two occurrences, not yet a
/// pattern worth sharing.
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
    use std::sync::{Arc, Mutex};

    use super::*;
    use crate::{
        CustomerContextId, CustomerId, InvoiceKind, MockProjectRepository, MockQuoteRepository,
        Organization, Quote, QuoteId, QuoteStatus, UserId,
    };
    use mockall::predicate::eq;
    use uuid::Uuid;

    fn line_command(quantity: Decimal, unit_price_cents: i32) -> InvoiceLineCommand {
        InvoiceLineCommand {
            label: "Acompte travaux".to_owned(),
            quantity,
            unit_price_cents,
            vat_rate_basis_points: None,
        }
    }

    fn invoice(id: InvoiceId) -> Invoice {
        let now = Utc::now();
        let organization_id = OrganizationId(Uuid::new_v4());
        Invoice {
            id,
            organization_id,
            number: None,
            kind: InvoiceKind::Standard,
            project_id: None,
            customer_id: CustomerId(Uuid::new_v4()),
            customer_context_id: CustomerContextId(Uuid::new_v4()),
            status: InvoiceStatus::Draft,
            issued_at: None,
            due_at: None,
            notes: None,
            operation_nature: None,
            delivery_address: None,
            net_cents: 5500,
            vat_breakdown: Vec::new(),
            gross_cents: 5500,
            issuer_identity: None,
            lines: vec![InvoiceLine {
                id: InvoiceLineId(Uuid::new_v4()),
                organization_id,
                invoice_id: id,
                label: "Acompte travaux".to_owned(),
                quantity: Decimal::new(1, 0),
                unit_price_cents: 5500,
                vat_rate_basis_points: None,
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

    fn organization_without_vat_status(id: OrganizationId) -> Organization {
        let now = Utc::now();
        Organization {
            id,
            name: "Acme".into(),
            slug: "acme".into(),
            owner_id: UserId(Uuid::new_v4()),
            legal_name: None,
            legal_form: None,
            registration_number: None,
            vat_status: None,
            share_capital_cents: None,
            address_line1: None,
            address_line2: None,
            address_postal_code: None,
            address_city: None,
            address_country: None,
            contact_email: None,
            contact_phone: None,
            insurance_mention: None,
            quote_number_prefix: "DEV".to_owned(),
            invoice_number_prefix: "FAC".to_owned(),
            field_clock_enabled: false,
            vat_on_debits: false,
            deleted_at: None,
            created_at: now,
            updated_at: now,
        }
    }

    fn organization_subject_to_vat(id: OrganizationId) -> Organization {
        Organization {
            vat_status: Some(VatStatus::Subject {
                vat_number: "FR12345678901".into(),
            }),
            ..organization_without_vat_status(id)
        }
    }

    fn mock_organization_repository(
        organization: Organization,
    ) -> crate::domain::organization::ports::MockOrganizationRepository {
        let mut repo = crate::domain::organization::ports::MockOrganizationRepository::new();
        repo.expect_find_by_id().times(1).returning(move |_| {
            let organization = organization.clone();
            Box::pin(async move { Ok(Some(organization)) })
        });
        repo
    }

    #[tokio::test]
    async fn create_invoice_calculates_totals_from_lines() {
        let organization_id = OrganizationId(Uuid::new_v4());
        let mut repo = crate::domain::invoice::ports::MockInvoiceRepository::new();
        repo.expect_insert_draft().times(1).returning(|d| {
            let invoice = d.invoice().clone();
            Box::pin(async move { Ok(invoice) })
        });

        let mut service = InvoiceService::new(repo, events::testing::RecordingEmitter::new());
        let created = service
            .create_invoice(
                CreateInvoiceCommand {
                    organization_id,
                    kind: InvoiceKind::Standard,
                    project_id: None,
                    customer_id: CustomerId(Uuid::new_v4()),
                    customer_context_id: CustomerContextId(Uuid::new_v4()),
                    due_at: None,
                    notes: None,
                    operation_nature: None,
                    delivery_address: None,
                    lines: vec![
                        line_command(Decimal::new(25, 1), 1200),
                        line_command(Decimal::new(1, 0), 500),
                    ],
                },
                mock_organization_repository(organization_without_vat_status(organization_id)),
            )
            .await
            .unwrap();

        assert_eq!(created.status, InvoiceStatus::Draft);
        assert_eq!(created.number, None, "a draft has no number");
        assert_eq!(created.net_cents, 3500);
        assert_eq!(created.gross_cents, 3500);
    }

    #[tokio::test]
    async fn create_invoice_breaks_vat_down_per_rate() {
        let organization_id = OrganizationId(Uuid::new_v4());
        let mut repo = crate::domain::invoice::ports::MockInvoiceRepository::new();
        repo.expect_insert_draft().times(1).returning(|d| {
            let invoice = d.invoice().clone();
            Box::pin(async move { Ok(invoice) })
        });

        let mut service = InvoiceService::new(repo, events::testing::RecordingEmitter::new());
        let mut reduced_rate = line_command(Decimal::new(1, 0), 10_000);
        reduced_rate.vat_rate_basis_points = Some(550);
        let mut standard_rate = line_command(Decimal::new(1, 0), 10_000);
        standard_rate.vat_rate_basis_points = Some(2000);

        let created = service
            .create_invoice(
                CreateInvoiceCommand {
                    organization_id,
                    kind: InvoiceKind::Standard,
                    project_id: None,
                    customer_id: CustomerId(Uuid::new_v4()),
                    customer_context_id: CustomerContextId(Uuid::new_v4()),
                    due_at: None,
                    notes: None,
                    operation_nature: None,
                    delivery_address: None,
                    lines: vec![reduced_rate, standard_rate],
                },
                mock_organization_repository(organization_subject_to_vat(organization_id)),
            )
            .await
            .unwrap();

        assert_eq!(created.net_cents, 20_000);
        assert_eq!(
            created.vat_breakdown,
            vec![
                InvoiceVatBreakdownLine {
                    rate_bp: 550,
                    vat_cents: 550
                },
                InvoiceVatBreakdownLine {
                    rate_bp: 2000,
                    vat_cents: 2000
                },
            ]
        );
        assert_eq!(created.gross_cents, 22_550);
    }

    #[tokio::test]
    async fn rejects_an_invoice_with_no_lines() {
        let organization_id = OrganizationId(Uuid::new_v4());
        let repo = crate::domain::invoice::ports::MockInvoiceRepository::new();
        let mut service = InvoiceService::new(repo, events::testing::RecordingEmitter::new());

        let result = service
            .create_invoice(
                CreateInvoiceCommand {
                    organization_id,
                    kind: InvoiceKind::Standard,
                    project_id: None,
                    customer_id: CustomerId(Uuid::new_v4()),
                    customer_context_id: CustomerContextId(Uuid::new_v4()),
                    due_at: None,
                    notes: None,
                    operation_nature: None,
                    delivery_address: None,
                    lines: vec![],
                },
                mock_organization_repository(organization_without_vat_status(organization_id)),
            )
            .await;

        assert!(matches!(result, Err(CoreError::Conflict(_))));
    }

    #[tokio::test]
    async fn update_invoice_recalculates_totals_on_a_draft() {
        let id = InvoiceId(Uuid::new_v4());
        let mut repo = crate::domain::invoice::ports::MockInvoiceRepository::new();
        repo.expect_find_by_id()
            .with(eq(id))
            .returning(move |_| Box::pin(async move { Ok(Some(invoice(id))) }));
        repo.expect_update_draft().times(1).returning(|d| {
            let invoice = d.invoice().clone();
            Box::pin(async move { Ok(invoice) })
        });

        let organization_id = invoice(id).organization_id;
        let mut service = InvoiceService::new(repo, events::testing::RecordingEmitter::new());
        let updated = service
            .update_invoice(
                UpdateInvoiceCommand {
                    id,
                    project_id: None,
                    customer_id: CustomerId(Uuid::new_v4()),
                    customer_context_id: CustomerContextId(Uuid::new_v4()),
                    due_at: None,
                    notes: Some("Merci".to_owned()),
                    operation_nature: None,
                    delivery_address: None,
                    lines: vec![line_command(Decimal::new(3, 0), 2000)],
                },
                mock_organization_repository(organization_without_vat_status(organization_id)),
            )
            .await
            .unwrap();

        assert_eq!(updated.net_cents, 6000);
        assert_eq!(updated.notes.as_deref(), Some("Merci"));
    }

    #[tokio::test]
    async fn update_invoice_refuses_an_issued_invoice() {
        let id = InvoiceId(Uuid::new_v4());
        let issued = Invoice {
            status: InvoiceStatus::Issued,
            number: Some("FAC-2026-0001".to_owned()),
            issued_at: Some(Utc::now()),
            ..invoice(id)
        };
        let mut repo = crate::domain::invoice::ports::MockInvoiceRepository::new();
        repo.expect_find_by_id().with(eq(id)).returning(move |_| {
            let issued = issued.clone();
            Box::pin(async move { Ok(Some(issued)) })
        });
        // No `expect_update_draft`: the call must never reach the
        // repository, it has to be refused before that.

        let organization_id = invoice(id).organization_id;
        let mut service = InvoiceService::new(repo, events::testing::RecordingEmitter::new());
        let result = service
            .update_invoice(
                UpdateInvoiceCommand {
                    id,
                    project_id: None,
                    customer_id: CustomerId(Uuid::new_v4()),
                    customer_context_id: CustomerContextId(Uuid::new_v4()),
                    due_at: None,
                    notes: None,
                    operation_nature: None,
                    delivery_address: None,
                    lines: vec![line_command(Decimal::new(1, 0), 1000)],
                },
                mock_organization_repository(organization_without_vat_status(organization_id)),
            )
            .await;

        assert!(matches!(result, Err(CoreError::Conflict(_))));
    }

    #[tokio::test]
    async fn cancel_invoice_transitions_a_draft() {
        let id = InvoiceId(Uuid::new_v4());
        let mut repo = crate::domain::invoice::ports::MockInvoiceRepository::new();
        repo.expect_find_by_id()
            .with(eq(id))
            .returning(move |_| Box::pin(async move { Ok(Some(invoice(id))) }));
        repo.expect_update_status()
            .withf(move |invoice_id, status, _| {
                *invoice_id == id && *status == InvoiceStatus::Cancelled
            })
            .returning(move |_, status, _| {
                let mut i = invoice(id);
                i.status = status;
                Box::pin(async move { Ok(i) })
            });

        let emitter = events::testing::RecordingEmitter::new();
        let mut service = InvoiceService::new(repo, &emitter);

        service
            .cancel_invoice(CancelInvoiceCommand { id })
            .await
            .unwrap();

        assert_eq!(emitter.names(), vec!["invoice.cancelled"]);
    }

    #[tokio::test]
    async fn cancel_invoice_refuses_an_already_cancelled_invoice() {
        let id = InvoiceId(Uuid::new_v4());
        let cancelled = Invoice {
            status: InvoiceStatus::Cancelled,
            ..invoice(id)
        };
        let mut repo = crate::domain::invoice::ports::MockInvoiceRepository::new();
        repo.expect_find_by_id().with(eq(id)).returning(move |_| {
            let cancelled = cancelled.clone();
            Box::pin(async move { Ok(Some(cancelled)) })
        });

        let mut service = InvoiceService::new(repo, events::testing::RecordingEmitter::new());
        let result = service.cancel_invoice(CancelInvoiceCommand { id }).await;

        assert!(matches!(result, Err(CoreError::Conflict(_))));
    }

    #[tokio::test]
    async fn soft_delete_refuses_anything_but_a_draft() {
        let id = InvoiceId(Uuid::new_v4());
        let issued = Invoice {
            status: InvoiceStatus::Issued,
            ..invoice(id)
        };
        let mut repo = crate::domain::invoice::ports::MockInvoiceRepository::new();
        repo.expect_find_by_id().with(eq(id)).returning(move |_| {
            let issued = issued.clone();
            Box::pin(async move { Ok(Some(issued)) })
        });
        // No `expect_soft_delete`: must be refused before the repository is
        // reached.

        let mut service = InvoiceService::new(repo, events::testing::RecordingEmitter::new());
        let result = service.soft_delete_invoice(id).await;

        assert!(matches!(result, Err(CoreError::Conflict(_))));
    }

    #[tokio::test]
    async fn soft_delete_deletes_a_draft() {
        let id = InvoiceId(Uuid::new_v4());
        let mut repo = crate::domain::invoice::ports::MockInvoiceRepository::new();
        repo.expect_find_by_id()
            .with(eq(id))
            .returning(move |_| Box::pin(async move { Ok(Some(invoice(id))) }));
        repo.expect_soft_delete()
            .withf(move |deleted_id, _| *deleted_id == id)
            .returning(|_, _| Box::pin(async { Ok(()) }));

        let emitter = events::testing::RecordingEmitter::new();
        let mut service = InvoiceService::new(repo, &emitter);

        service.soft_delete_invoice(id).await.unwrap();

        assert_eq!(emitter.names(), vec!["invoice.deleted"]);
    }

    /// The distinguishing rounding case, mirrored from
    /// `quote::service::three_half_cent_vat_lines_round_per_line_then_sum`.
    #[tokio::test]
    async fn three_half_cent_vat_lines_round_per_line_then_sum() {
        let organization_id = OrganizationId(Uuid::new_v4());
        let mut repo = crate::domain::invoice::ports::MockInvoiceRepository::new();
        repo.expect_insert_draft().times(1).returning(|d| {
            let invoice = d.invoice().clone();
            Box::pin(async move { Ok(invoice) })
        });

        let mut service = InvoiceService::new(repo, events::testing::RecordingEmitter::new());
        let mut line = line_command(Decimal::new(1, 0), 105);
        line.vat_rate_basis_points = Some(1000);

        let created = service
            .create_invoice(
                CreateInvoiceCommand {
                    organization_id,
                    kind: InvoiceKind::Standard,
                    project_id: None,
                    customer_id: CustomerId(Uuid::new_v4()),
                    customer_context_id: CustomerContextId(Uuid::new_v4()),
                    due_at: None,
                    notes: None,
                    operation_nature: None,
                    delivery_address: None,
                    lines: vec![line.clone(), line.clone(), line],
                },
                mock_organization_repository(organization_subject_to_vat(organization_id)),
            )
            .await
            .unwrap();

        assert_eq!(created.net_cents, 315);
        assert_eq!(
            created.vat_breakdown,
            vec![InvoiceVatBreakdownLine {
                rate_bp: 1000,
                vat_cents: 30
            }]
        );
        assert_eq!(created.gross_cents, 345);
    }

    fn complete_organization(id: OrganizationId) -> Organization {
        Organization {
            legal_name: Some("Acme SARL".into()),
            legal_form: Some("SARL".into()),
            registration_number: Some("123 456 789 00012".into()),
            vat_status: Some(VatStatus::Subject {
                vat_number: "FR12345678901".into(),
            }),
            share_capital_cents: Some(1_000_000),
            address_line1: Some("12 rue des Artisans".into()),
            address_postal_code: Some("75001".into()),
            address_city: Some("Paris".into()),
            address_country: Some("FR".into()),
            insurance_mention: Some("RC Pro n°123456 - MAAF Assurances".into()),
            ..organization_without_vat_status(id)
        }
    }

    fn quote_with_net_cents(
        id: QuoteId,
        organization_id: OrganizationId,
        customer_id: CustomerId,
        customer_context_id: CustomerContextId,
        net_cents: i32,
    ) -> Quote {
        let now = Utc::now();
        Quote {
            id,
            organization_id,
            reference: Some("DEV-2026-0001".to_owned()),
            title: "Chantier".to_owned(),
            customer_id,
            customer_context_id,
            status: QuoteStatus::Accepted,
            net_cents,
            vat_breakdown: Vec::new(),
            gross_cents: net_cents,
            lines: Vec::new(),
            deleted_at: None,
            created_at: now,
            updated_at: now,
        }
    }

    fn project_with_quote(
        organization_id: OrganizationId,
        quote_id: QuoteId,
        customer_id: CustomerId,
        customer_context_id: CustomerContextId,
    ) -> Project {
        let now = Utc::now();
        Project {
            id: ProjectId(Uuid::new_v4()),
            organization_id,
            name: "Chantier".to_owned(),
            customer_id: Some(customer_id),
            customer_context_id: Some(customer_context_id),
            quote_id: Some(quote_id),
            archived_at: None,
            created_at: now,
            updated_at: now,
        }
    }

    fn project_without_quote(organization_id: OrganizationId) -> Project {
        let now = Utc::now();
        Project {
            id: ProjectId(Uuid::new_v4()),
            organization_id,
            name: "Chantier".to_owned(),
            customer_id: Some(CustomerId(Uuid::new_v4())),
            customer_context_id: Some(CustomerContextId(Uuid::new_v4())),
            quote_id: None,
            archived_at: None,
            created_at: now,
            updated_at: now,
        }
    }

    fn issued_invoice_stub(
        organization_id: OrganizationId,
        project_id: ProjectId,
        net_cents: i32,
    ) -> Invoice {
        Invoice {
            status: InvoiceStatus::Issued,
            number: Some("FAC-2026-0001".to_owned()),
            issued_at: Some(Utc::now()),
            project_id: Some(project_id),
            net_cents,
            gross_cents: net_cents,
            organization_id,
            ..invoice(InvoiceId(Uuid::new_v4()))
        }
    }

    fn project_repo_with(project: Project) -> MockProjectRepository {
        let mut repo = MockProjectRepository::new();
        repo.expect_find_by_id().returning(move |_| {
            let project = project.clone();
            Box::pin(async move { Ok(Some(project)) })
        });
        repo
    }

    fn quote_repo_with(quote: Quote) -> MockQuoteRepository {
        let mut repo = MockQuoteRepository::new();
        repo.expect_find_by_id().returning(move |_| {
            let quote = quote.clone();
            Box::pin(async move { Ok(Some(quote)) })
        });
        repo
    }

    /// A fake `InvoiceRepository` over a shared, growing store — needed
    /// because the rounding-invariant test below issues three invoices in
    /// sequence against the same project and each one has to see what the
    /// previous ones actually landed at, not a value the test precomputed
    /// independently.
    fn invoice_repository_over(
        store: Arc<Mutex<Vec<Invoice>>>,
    ) -> crate::domain::invoice::ports::MockInvoiceRepository {
        let mut repo = crate::domain::invoice::ports::MockInvoiceRepository::new();

        let list_store = store.clone();
        repo.expect_list_by_project().returning(move |_| {
            let invoices = list_store.lock().unwrap().clone();
            Box::pin(async move { Ok(invoices) })
        });

        let insert_store = store.clone();
        repo.expect_insert_draft().returning(move |draft| {
            let invoice = draft.invoice().clone();
            insert_store.lock().unwrap().push(invoice.clone());
            Box::pin(async move { Ok(invoice) })
        });

        let mut next_number = 0u32;
        repo.expect_allocate_number()
            .returning(move |_, prefix, year| {
                next_number += 1;
                let number = format!("{prefix}-{year}-{next_number:04}");
                Box::pin(async move { Ok(number) })
            });

        let issue_store = store.clone();
        repo.expect_issue()
            .returning(move |id, number, issued_at, issuer_identity, updated_at| {
                let mut invoices = issue_store.lock().unwrap();
                let invoice = invoices
                    .iter_mut()
                    .find(|invoice| invoice.id == id)
                    .expect("issue called on an invoice that was never inserted");
                invoice.number = Some(number);
                invoice.status = InvoiceStatus::Issued;
                invoice.issued_at = Some(issued_at);
                invoice.issuer_identity = Some(issuer_identity.clone());
                invoice.updated_at = updated_at;
                let result = invoice.clone();
                Box::pin(async move { Ok(result) })
            });

        repo
    }

    /// The rounding invariant this issue exists to protect: 100_007 divides
    /// evenly by neither 10 nor 3, so a 30% deposit cannot land on a whole
    /// number of cents (30% of 100_007 is 30_002.1) and genuinely exercises
    /// rounding rather than getting lucky with a round quote. Two such
    /// deposits round independently, but the final invoice's amount is not
    /// its own independent percentage — it is exactly what
    /// `remaining_to_bill_cents` says is left — so whatever the deposits'
    /// rounding cost or gained, the three invoices sum to the quote's
    /// `net_cents` exactly, to the cent.
    #[tokio::test]
    async fn deposits_then_final_invoice_round_to_a_zero_remainder() {
        let organization_id = OrganizationId(Uuid::new_v4());
        let quote_id = QuoteId(Uuid::new_v4());
        let customer_id = CustomerId(Uuid::new_v4());
        let customer_context_id = CustomerContextId(Uuid::new_v4());
        let quoted_cents = 100_007;

        let project =
            project_with_quote(organization_id, quote_id, customer_id, customer_context_id);
        let quote = quote_with_net_cents(
            quote_id,
            organization_id,
            customer_id,
            customer_context_id,
            quoted_cents,
        );
        let organization = complete_organization(organization_id);

        let store: Arc<Mutex<Vec<Invoice>>> = Arc::new(Mutex::new(Vec::new()));
        let mut service = InvoiceService::new(
            invoice_repository_over(store.clone()),
            events::testing::RecordingEmitter::new(),
        );

        let deposit_command = || IssueDepositCommand {
            project_id: project.id,
            percentage_bp: 3000,
            due_at: None,
            notes: None,
            allow_exceeding_total: false,
        };

        let deposit1 = service
            .issue_deposit(
                deposit_command(),
                mock_organization_repository(organization.clone()),
                project_repo_with(project.clone()),
                quote_repo_with(quote.clone()),
            )
            .await
            .unwrap();

        let deposit2 = service
            .issue_deposit(
                deposit_command(),
                mock_organization_repository(organization.clone()),
                project_repo_with(project.clone()),
                quote_repo_with(quote.clone()),
            )
            .await
            .unwrap();

        let final_invoice = service
            .issue_final_invoice(
                IssueFinalInvoiceCommand {
                    project_id: project.id,
                    due_at: None,
                    notes: None,
                    allow_exceeding_total: false,
                },
                mock_organization_repository(organization),
                project_repo_with(project.clone()),
                quote_repo_with(quote),
            )
            .await
            .unwrap();

        assert_eq!(deposit1.net_cents, 30_002);
        assert_eq!(deposit2.net_cents, 30_002);
        assert_eq!(final_invoice.net_cents, 40_003);
        assert_eq!(
            deposit1.net_cents + deposit2.net_cents + final_invoice.net_cents,
            quoted_cents,
            "the remainder after every invoice must round to exactly zero"
        );
        assert_eq!(deposit1.kind, InvoiceKind::Deposit);
        assert_eq!(final_invoice.kind, InvoiceKind::Final);
    }

    #[tokio::test]
    async fn issue_deposit_refuses_to_exceed_the_quoted_total_unless_allowed() {
        let organization_id = OrganizationId(Uuid::new_v4());
        let quote_id = QuoteId(Uuid::new_v4());
        let customer_id = CustomerId(Uuid::new_v4());
        let customer_context_id = CustomerContextId(Uuid::new_v4());
        let quoted_cents = 10_000;

        let project =
            project_with_quote(organization_id, quote_id, customer_id, customer_context_id);
        let quote = quote_with_net_cents(
            quote_id,
            organization_id,
            customer_id,
            customer_context_id,
            quoted_cents,
        );
        let organization = complete_organization(organization_id);

        // 90% already issued; an 80% deposit on top would reach 170%.
        let store: Arc<Mutex<Vec<Invoice>>> = Arc::new(Mutex::new(vec![issued_invoice_stub(
            organization_id,
            project.id,
            9_000,
        )]));
        let mut service = InvoiceService::new(
            invoice_repository_over(store.clone()),
            events::testing::RecordingEmitter::new(),
        );

        let refused = service
            .issue_deposit(
                IssueDepositCommand {
                    project_id: project.id,
                    percentage_bp: 8000,
                    due_at: None,
                    notes: None,
                    allow_exceeding_total: false,
                },
                // Never reached: the pre-check refuses before the
                // organization is ever resolved.
                crate::domain::organization::ports::MockOrganizationRepository::new(),
                project_repo_with(project.clone()),
                quote_repo_with(quote.clone()),
            )
            .await;

        assert!(matches!(refused, Err(CoreError::Conflict(_))));
        assert_eq!(
            store.lock().unwrap().len(),
            1,
            "a refused deposit must not leave a half-created draft behind"
        );

        let allowed = service
            .issue_deposit(
                IssueDepositCommand {
                    project_id: project.id,
                    percentage_bp: 8000,
                    due_at: None,
                    notes: None,
                    allow_exceeding_total: true,
                },
                mock_organization_repository(organization),
                project_repo_with(project.clone()),
                quote_repo_with(quote),
            )
            .await
            .unwrap();

        assert_eq!(allowed.status, InvoiceStatus::Issued);
    }

    #[tokio::test]
    async fn issue_invoice_refuses_an_incomplete_organization_legal_identity() {
        let id = InvoiceId(Uuid::new_v4());
        let organization_id = OrganizationId(Uuid::new_v4());
        let draft = Invoice {
            organization_id,
            ..invoice(id)
        };

        let mut repo = crate::domain::invoice::ports::MockInvoiceRepository::new();
        repo.expect_find_by_id().with(eq(id)).returning(move |_| {
            let draft = draft.clone();
            Box::pin(async move { Ok(Some(draft)) })
        });
        // No `expect_allocate_number`/`expect_issue`: the call must never
        // reach them, refused before that on the legal identity alone.

        let mut service = InvoiceService::new(repo, events::testing::RecordingEmitter::new());

        let result = service
            .issue_invoice(
                IssueInvoiceCommand {
                    id,
                    allow_exceeding_total: false,
                },
                mock_organization_repository(organization_without_vat_status(organization_id)),
                MockProjectRepository::new(),
                MockQuoteRepository::new(),
            )
            .await;

        assert!(matches!(result, Err(CoreError::Conflict(_))));
    }

    #[tokio::test]
    async fn issue_deposit_refuses_a_project_with_no_quote() {
        let organization_id = OrganizationId(Uuid::new_v4());
        let project = project_without_quote(organization_id);
        let project_id = project.id;

        let repo = crate::domain::invoice::ports::MockInvoiceRepository::new();
        let mut service = InvoiceService::new(repo, events::testing::RecordingEmitter::new());

        let result = service
            .issue_deposit(
                IssueDepositCommand {
                    project_id,
                    percentage_bp: 3000,
                    due_at: None,
                    notes: None,
                    allow_exceeding_total: false,
                },
                crate::domain::organization::ports::MockOrganizationRepository::new(),
                project_repo_with(project),
                MockQuoteRepository::new(),
            )
            .await;

        assert!(matches!(result, Err(CoreError::Conflict(_))));
    }

    #[tokio::test]
    async fn issue_final_invoice_refuses_a_project_with_no_quote() {
        let organization_id = OrganizationId(Uuid::new_v4());
        let project = project_without_quote(organization_id);
        let project_id = project.id;

        let repo = crate::domain::invoice::ports::MockInvoiceRepository::new();
        let mut service = InvoiceService::new(repo, events::testing::RecordingEmitter::new());

        let result = service
            .issue_final_invoice(
                IssueFinalInvoiceCommand {
                    project_id,
                    due_at: None,
                    notes: None,
                    allow_exceeding_total: false,
                },
                crate::domain::organization::ports::MockOrganizationRepository::new(),
                project_repo_with(project),
                MockQuoteRepository::new(),
            )
            .await;

        assert!(matches!(result, Err(CoreError::Conflict(_))));
    }

    /// Nothing about `create_invoice`/`issue_invoice` requires a project or
    /// a quote — that requirement is specific to `issue_deposit` and
    /// `issue_final_invoice`, which need a total to compute a percentage or
    /// a remainder against. A manually created, project-less draft issues
    /// exactly like one with a quote-backed project, minus the over-issuance
    /// check (nothing to compare against).
    #[tokio::test]
    async fn issue_invoice_does_not_require_a_project_or_a_quote() {
        let id = InvoiceId(Uuid::new_v4());
        let organization_id = OrganizationId(Uuid::new_v4());
        let draft = Invoice {
            organization_id,
            project_id: None,
            ..invoice(id)
        };

        let mut repo = crate::domain::invoice::ports::MockInvoiceRepository::new();
        repo.expect_find_by_id().with(eq(id)).returning(move |_| {
            let draft = draft.clone();
            Box::pin(async move { Ok(Some(draft)) })
        });
        repo.expect_allocate_number().returning(|_, prefix, year| {
            let number = format!("{prefix}-{year}-0001");
            Box::pin(async move { Ok(number) })
        });
        repo.expect_issue().returning(
            move |issued_id, number, issued_at, issuer_identity, updated_at| {
                let mut issued = invoice(issued_id);
                issued.project_id = None;
                issued.status = InvoiceStatus::Issued;
                issued.number = Some(number);
                issued.issued_at = Some(issued_at);
                issued.issuer_identity = Some(issuer_identity.clone());
                issued.updated_at = updated_at;
                Box::pin(async move { Ok(issued) })
            },
        );

        let mut service = InvoiceService::new(repo, events::testing::RecordingEmitter::new());

        let issued = service
            .issue_invoice(
                IssueInvoiceCommand {
                    id,
                    allow_exceeding_total: false,
                },
                mock_organization_repository(complete_organization(organization_id)),
                // Never reached: the invoice carries no project.
                MockProjectRepository::new(),
                MockQuoteRepository::new(),
            )
            .await
            .unwrap();

        assert_eq!(issued.status, InvoiceStatus::Issued);
        assert!(issued.number.is_some());
    }
}
