use authz::{Authorizer, Resource, Subject};
use chrono::{DateTime, Datelike, NaiveDate, Utc};
use common::{CoreError, UserId, generate_uuid_v7};
use events::EventEmitter;
use rust_decimal::Decimal;
use serde_json::{Value, json};

use crate::{
    Customer, CustomerContextId, CustomerId, CustomerOutstandingBalance, DepositPercent,
    DraftInvoice, GeneratedInvoiceDocument, Invoice, InvoiceCollection, InvoiceDerived,
    InvoiceEvent, InvoiceEventId, InvoiceEventKind, InvoiceId, InvoiceKind, InvoiceLine,
    InvoiceLineId, InvoiceNumberSeries, InvoicePayment, InvoicePaymentId, InvoiceSettlement,
    InvoiceStatus, LegalIdentity, Organization, OrganizationId, Project, ProjectBillingSummary,
    ProjectId, Quote, QuoteId, RecipientSnapshot, VatStatus,
    application::policy,
    domain::{
        customer::ports::CustomerRepository,
        customer_context::ports::CustomerContextRepository,
        invoice::{
            InvoiceError, InvoiceField, InvoiceOutcome,
            commands::{
                CancelInvoiceCommand, CreateInvoiceCommand, CreateInvoiceFromQuoteCommand,
                DeleteInvoicePaymentCommand, InvoiceLineCommand, IssueCreditNoteCommand,
                IssueDepositCommand, IssueFinalInvoiceCommand, IssueInvoiceCommand,
                NoteInvoiceCommand, QuoteInvoiceMode, RecordInvoicePaymentCommand,
                RemindInvoiceCommand, SendInvoiceCommand, UpdateInvoiceCommand,
            },
            events::{
                InvoiceCreated, InvoiceDeleted, InvoicePaymentDeleted, InvoicePaymentRecorded,
                InvoiceTransitioned, InvoiceUpdated,
            },
            ports::InvoiceRepository,
            rules::{
                self, DocumentLines, IssueFacts, QuoteInvoicing, check_credit_note_date,
                check_payment, credit_note_lines, deposit_lines, discount_of, display_state,
                due_at_for, full_invoice_lines, fully_credits, is_active_on_quote,
                legal_notice_for, quote_invoicing, reminder_detail, settlement_of, totals_of,
                validate_issue, validate_line,
            },
        },
        member::ports::MemberRepository,
        organization::{
            billing_settings::{BillingSettings, IssuerBilling, PaymentTermsDays},
            numbering::CREDIT_NOTE_NUMBER_PREFIX,
            ports::OrganizationRepository,
        },
        project::{ports::ProjectRepository, service::remaining_to_bill_cents},
        quote::ports::QuoteRepository,
        role::ports::RoleRepository,
    },
};

enum Stop {
    Core(CoreError),
    Refused(InvoiceError),
}

impl From<CoreError> for Stop {
    fn from(error: CoreError) -> Self {
        Self::Core(error)
    }
}

impl From<InvoiceError> for Stop {
    fn from(error: InvoiceError) -> Self {
        Self::Refused(error)
    }
}

fn settle<T>(result: Result<T, Stop>) -> Result<InvoiceOutcome<T>, CoreError> {
    match result {
        Ok(value) => Ok(Ok(value)),
        Err(Stop::Core(error)) => Err(error),
        Err(Stop::Refused(refusal)) => Ok(Err(refusal)),
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QuoteInvoicingView {
    pub organization_id: OrganizationId,
    pub invoicing: QuoteInvoicing,
    pub invoices: Vec<Invoice>,
}

struct DraftTerms {
    payment_terms_days: i32,
    legal_notice: String,
}

struct Issuable {
    organization: Organization,
    settings: BillingSettings,
    legal_identity: LegalIdentity,
}

struct SingleLineDraft {
    invoice_id: InvoiceId,
    organization_id: OrganizationId,
    kind: InvoiceKind,
    project_id: ProjectId,
    customer_id: CustomerId,
    customer_context_id: CustomerContextId,
    notes: Option<String>,
    label: String,
    amount_cents: i32,
    payment_terms_days: Option<i64>,
    now: DateTime<Utc>,
}

pub struct InvoiceService<R, E, M, Ro, A>
where
    R: InvoiceRepository,
    E: EventEmitter,
    M: MemberRepository,
    Ro: RoleRepository,
    A: Authorizer,
{
    repo: R,
    emitter: E,
    member_repository: M,
    role_repository: Ro,
    authz: A,
}

impl<R, E, M, Ro, A> InvoiceService<R, E, M, Ro, A>
where
    R: InvoiceRepository,
    E: EventEmitter,
    M: MemberRepository,
    Ro: RoleRepository,
    A: Authorizer,
{
    pub fn new(repo: R, emitter: E, member_repository: M, role_repository: Ro, authz: A) -> Self {
        Self {
            repo,
            emitter,
            member_repository,
            role_repository,
            authz,
        }
    }

    async fn require_manage_invoices(
        &mut self,
        actor: Subject,
        organization_id: OrganizationId,
    ) -> Result<(), CoreError> {
        let actor = policy::enrich_for_organization(
            actor,
            organization_id,
            &mut self.member_repository,
            &mut self.role_repository,
        )
        .await?;
        policy::require(
            &self.authz,
            &actor,
            "invoice.manage",
            Resource::new("organization", organization_id.0.to_string()),
        )
        .await
    }

    async fn load(&mut self, id: InvoiceId) -> Result<Invoice, CoreError> {
        self.repo.find_by_id(id).await?.ok_or(CoreError::NotFound)
    }

    async fn decorate_many(
        &mut self,
        invoices: Vec<Invoice>,
        today: NaiveDate,
    ) -> Result<Vec<Invoice>, CoreError> {
        let ids: Vec<InvoiceId> = invoices.iter().map(|invoice| invoice.id).collect();
        let collections = self.repo.collections(&ids).await?;

        Ok(invoices
            .into_iter()
            .map(|invoice| {
                let collection = collections
                    .iter()
                    .find(|collection| collection.invoice_id == invoice.id);
                decorate(invoice, collection, today)
            })
            .collect())
    }

    async fn decorate_one(
        &mut self,
        invoice: Invoice,
        today: NaiveDate,
    ) -> Result<Invoice, CoreError> {
        let mut decorated = self.decorate_many(vec![invoice], today).await?;

        Ok(decorated.remove(0))
    }

    pub async fn get_invoice(
        &mut self,
        id: InvoiceId,
        today: NaiveDate,
    ) -> Result<Invoice, CoreError> {
        let invoice = self.load(id).await?;
        let mut decorated = self.decorate_one(invoice, today).await?;
        decorated.derived.events = self.repo.list_events(id).await?;

        Ok(decorated)
    }

    pub async fn list_invoices(
        &mut self,
        organization_id: OrganizationId,
        limit: u64,
        offset: u64,
        today: NaiveDate,
    ) -> Result<(Vec<Invoice>, u64), CoreError> {
        let (invoices, total) = self
            .repo
            .list_by_organization(organization_id, limit, offset)
            .await?;

        Ok((self.decorate_many(invoices, today).await?, total))
    }

    pub async fn list_invoices_by_project(
        &mut self,
        project_id: ProjectId,
        today: NaiveDate,
    ) -> Result<Vec<Invoice>, CoreError> {
        let invoices = self.repo.list_by_project(project_id).await?;

        self.decorate_many(invoices, today).await
    }

    pub async fn get_invoice_credit_notes(
        &mut self,
        source_invoice_id: InvoiceId,
        today: NaiveDate,
    ) -> Result<Vec<Invoice>, CoreError> {
        let credit_notes = self.repo.list_by_source_invoice(source_invoice_id).await?;

        self.decorate_many(credit_notes, today).await
    }

    pub async fn create_invoice<O, C>(
        &mut self,
        command: CreateInvoiceCommand,
        organizations: &mut O,
        customers: &mut C,
        now: DateTime<Utc>,
    ) -> Result<InvoiceOutcome<Invoice>, CoreError>
    where
        O: OrganizationRepository,
        C: CustomerRepository,
    {
        self.require_manage_invoices(command.actor.clone(), command.organization_id)
            .await?;

        settle(self.create(command, organizations, customers, now).await)
    }

    async fn create<O, C>(
        &mut self,
        command: CreateInvoiceCommand,
        organizations: &mut O,
        customers: &mut C,
        now: DateTime<Utc>,
    ) -> Result<Invoice, Stop>
    where
        O: OrganizationRepository,
        C: CustomerRepository,
    {
        let invoice_id = InvoiceId(generate_uuid_v7());
        let organization = resolve_organization(organizations, command.organization_id).await?;
        let customer =
            require_customer(customers, command.organization_id, command.customer_id).await?;
        let settings = organizations
            .find_billing_settings(command.organization_id)
            .await?
            .unwrap_or_default();
        let terms = draft_terms(
            command.payment_terms_days,
            command.legal_notice,
            &customer,
            &settings,
        )?;
        let lines = build_invoice_lines(command.organization_id, invoice_id, command.lines, now)?;
        let pricing = totals_of(&lines, is_subject_to_vat(&organization))?;

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
            due_at: None,
            notes: command.notes,
            operation_nature: command.operation_nature,
            delivery_address: command.delivery_address,
            net_cents: pricing.net_cents,
            vat_breakdown: pricing.vat_breakdown,
            gross_cents: pricing.gross_cents,
            issuer_identity: None,
            generated_document: None,
            lines,
            source_invoice_id: None,
            title: command.title.trim().to_owned(),
            payment_terms_days: terms.payment_terms_days,
            legal_notice: terms.legal_notice,
            internal_note: None,
            sent_at: None,
            quote_id: None,
            issued_on: None,
            reason: None,
            full_cancellation: false,
            derived: InvoiceDerived::default(),
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

    pub async fn update_invoice<O>(
        &mut self,
        command: UpdateInvoiceCommand,
        organizations: &mut O,
        now: DateTime<Utc>,
    ) -> Result<InvoiceOutcome<Invoice>, CoreError>
    where
        O: OrganizationRepository,
    {
        let existing = self.load(command.id).await?;
        self.require_manage_invoices(command.actor.clone(), existing.organization_id)
            .await?;

        settle(self.update(existing, command, organizations, now).await)
    }

    async fn update<O>(
        &mut self,
        existing: Invoice,
        command: UpdateInvoiceCommand,
        organizations: &mut O,
        now: DateTime<Utc>,
    ) -> Result<Invoice, Stop>
    where
        O: OrganizationRepository,
    {
        if existing.status != InvoiceStatus::Draft {
            return Err(InvoiceError::Locked.into());
        }

        let organization = resolve_organization(organizations, existing.organization_id).await?;
        let terms = match command.payment_terms_days {
            Some(days) => payment_terms_of(days)?,
            None => existing.payment_terms_days,
        };
        let lines = build_invoice_lines(existing.organization_id, existing.id, command.lines, now)?;
        let pricing = totals_of(&lines, is_subject_to_vat(&organization))?;

        let mut draft = DraftInvoice::try_from_invoice(existing.clone())?;
        draft.set_project(command.project_id);
        draft.set_customer(command.customer_id, command.customer_context_id);
        draft.set_title(
            command
                .title
                .map_or_else(|| existing.title.clone(), |title| title.trim().to_owned()),
        );
        draft.set_payment_terms_days(terms);
        draft.set_legal_notice(
            normalized(command.legal_notice).unwrap_or_else(|| existing.legal_notice.clone()),
        );
        draft.set_notes(command.notes);
        draft.set_operation_nature(command.operation_nature);
        draft.set_delivery_address(command.delivery_address);
        draft.set_lines_and_totals(
            lines,
            pricing.net_cents,
            pricing.vat_breakdown,
            pricing.gross_cents,
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

    pub async fn cancel_invoice(
        &mut self,
        command: CancelInvoiceCommand,
    ) -> Result<Invoice, CoreError> {
        let existing = self.load(command.id).await?;
        self.require_manage_invoices(command.actor, existing.organization_id)
            .await?;

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

    pub async fn record_generated_document(
        &mut self,
        id: InvoiceId,
        document: GeneratedInvoiceDocument,
        actor: Subject,
    ) -> Result<Invoice, CoreError> {
        let existing = self.load(id).await?;
        self.require_manage_invoices(actor, existing.organization_id)
            .await?;

        if existing.status == InvoiceStatus::Draft {
            return Err(CoreError::Conflict(format!(
                "invoice {id} is still a draft; only an issued invoice can have a generated document"
            )));
        }
        if existing.generated_document.is_some() {
            return Err(CoreError::Conflict(format!(
                "invoice {id} already has a generated document"
            )));
        }

        self.repo
            .record_generated_document(id, &document, Utc::now())
            .await
    }

    pub async fn soft_delete_invoice(
        &mut self,
        id: InvoiceId,
        actor: Subject,
        now: DateTime<Utc>,
    ) -> Result<InvoiceOutcome<()>, CoreError> {
        let existing = self.load(id).await?;
        self.require_manage_invoices(actor, existing.organization_id)
            .await?;

        if existing.status != InvoiceStatus::Draft {
            return Ok(Err(InvoiceError::Locked));
        }

        self.repo.soft_delete(id, now).await?;
        self.emitter
            .emit(existing.organization_id, &InvoiceDeleted { invoice_id: id })?;

        Ok(Ok(()))
    }

    pub async fn record_payment(
        &mut self,
        command: RecordInvoicePaymentCommand,
        now: DateTime<Utc>,
    ) -> Result<InvoiceOutcome<InvoicePayment>, CoreError> {
        let invoice = self.load(command.invoice_id).await?;
        self.require_manage_invoices(command.actor.clone(), invoice.organization_id)
            .await?;

        settle(self.pay(invoice, command, now).await)
    }

    async fn pay(
        &mut self,
        invoice: Invoice,
        command: RecordInvoicePaymentCommand,
        now: DateTime<Utc>,
    ) -> Result<InvoicePayment, Stop> {
        if invoice.status != InvoiceStatus::Issued {
            return Err(InvoiceError::NotIssued.into());
        }

        let collection = self.repo.collections(&[invoice.id]).await?;
        let settlement = settlement_of_collection(&invoice, collection.first());
        check_payment(
            command.amount_cents,
            command.paid_on,
            now.date_naive(),
            settlement,
        )?;

        let payment = InvoicePayment {
            id: InvoicePaymentId(generate_uuid_v7()),
            organization_id: invoice.organization_id,
            invoice_id: invoice.id,
            amount_cents: command.amount_cents,
            paid_on: command.paid_on,
            method: command.method,
            reference: normalized(command.reference),
            note: normalized(command.note),
            recorded_by: command.recorded_by,
            deleted_at: None,
            deleted_by: None,
            created_at: now,
            updated_at: now,
        };

        let inserted = self.repo.insert_payment(&payment).await?;

        self.emitter.emit(
            inserted.organization_id,
            &InvoicePaymentRecorded {
                payment: inserted.clone(),
            },
        )?;

        Ok(inserted)
    }

    pub async fn delete_payment(
        &mut self,
        command: DeleteInvoicePaymentCommand,
    ) -> Result<(), CoreError> {
        let payment = self
            .repo
            .find_payment_by_id(command.id)
            .await?
            .ok_or(CoreError::NotFound)?;
        self.require_manage_invoices(command.actor, payment.organization_id)
            .await?;

        let deleted_at = Utc::now();
        self.repo
            .soft_delete_payment(command.id, deleted_at, command.deleted_by)
            .await?;

        self.emitter.emit(
            payment.organization_id,
            &InvoicePaymentDeleted {
                payment_id: command.id,
                invoice_id: payment.invoice_id,
                deleted_by: command.deleted_by,
            },
        )
    }

    pub async fn list_payments(
        &mut self,
        invoice_id: InvoiceId,
    ) -> Result<Vec<InvoicePayment>, CoreError> {
        self.repo.list_payments(invoice_id).await
    }

    pub async fn find_payment_by_id(
        &mut self,
        id: InvoicePaymentId,
    ) -> Result<Option<InvoicePayment>, CoreError> {
        self.repo.find_payment_by_id(id).await
    }

    pub async fn outstanding_by_customer(
        &mut self,
        organization_id: OrganizationId,
    ) -> Result<Vec<CustomerOutstandingBalance>, CoreError> {
        self.repo
            .list_outstanding_by_customer(organization_id)
            .await
    }

    pub async fn overdue_invoices(
        &mut self,
        organization_id: OrganizationId,
        as_of: DateTime<Utc>,
    ) -> Result<Vec<Invoice>, CoreError> {
        let invoices = self.repo.list_overdue(organization_id, as_of).await?;

        self.decorate_many(invoices, as_of.date_naive()).await
    }

    pub async fn project_billing_summary<P, Q>(
        &mut self,
        project_id: ProjectId,
        mut project_repository: P,
        mut quote_repository: Q,
    ) -> Result<ProjectBillingSummary, CoreError>
    where
        P: ProjectRepository,
        Q: QuoteRepository,
    {
        let project = project_repository
            .find_by_id(project_id)
            .await?
            .ok_or(CoreError::NotFound)?;

        let billed_cents = self.sum_already_issued_cents(project_id).await?;

        let quoted_cents = match project.quote_id {
            Some(quote_id) => {
                let quote = quote_repository
                    .find_by_id(quote_id)
                    .await?
                    .ok_or(CoreError::NotFound)?;
                Some(quote.net_cents)
            }
            None => None,
        };

        let remaining_cents = quoted_cents
            .map(|quoted| remaining_to_bill_cents(quoted, billed_cents))
            .transpose()?;

        Ok(ProjectBillingSummary {
            project_id,
            quoted_cents,
            billed_cents,
            remaining_cents,
        })
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn issue_invoice<O, P, Q, C, X>(
        &mut self,
        command: IssueInvoiceCommand,
        organizations: &mut O,
        projects: &mut P,
        quotes: &mut Q,
        customers: &mut C,
        contexts: &mut X,
        now: DateTime<Utc>,
    ) -> Result<InvoiceOutcome<Invoice>, CoreError>
    where
        O: OrganizationRepository,
        P: ProjectRepository,
        Q: QuoteRepository,
        C: CustomerRepository,
        X: CustomerContextRepository,
    {
        let existing = self.load(command.id).await?;
        self.require_manage_invoices(command.actor.clone(), existing.organization_id)
            .await?;

        let today = now.date_naive();
        let issue_date = command.issued_on.unwrap_or(today);
        let issued = async {
            let ready = self
                .check_issuable(
                    &existing,
                    issue_date,
                    today,
                    organizations,
                    customers,
                    contexts,
                )
                .await?;
            self.perform_issue(
                existing,
                ready,
                issue_date,
                command.allow_exceeding_total,
                projects,
                quotes,
                now,
            )
            .await
        }
        .await;

        settle(issued)
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn issue_deposit<O, P, Q, C, X>(
        &mut self,
        command: IssueDepositCommand,
        organizations: &mut O,
        projects: &mut P,
        quotes: &mut Q,
        customers: &mut C,
        contexts: &mut X,
        now: DateTime<Utc>,
    ) -> Result<InvoiceOutcome<Invoice>, CoreError>
    where
        O: OrganizationRepository,
        P: ProjectRepository,
        Q: QuoteRepository,
        C: CustomerRepository,
        X: CustomerContextRepository,
    {
        validate_percentage_bp(command.percentage_bp)?;

        let project = projects
            .find_by_id(command.project_id)
            .await?
            .ok_or(CoreError::NotFound)?;
        self.require_manage_invoices(command.actor.clone(), project.organization_id)
            .await?;
        let quote_id = project.quote_id.ok_or_else(|| {
            CoreError::Conflict(format!(
                "project {} has no quote; a deposit needs a total to compute a percentage against",
                project.id
            ))
        })?;
        let (customer_id, customer_context_id) = required_customer_fields(&project)?;
        let quote = quotes
            .find_by_id(quote_id)
            .await?
            .ok_or(CoreError::NotFound)?;

        let amount_cents = deposit_amount_cents(command.percentage_bp, quote.net_cents)?;

        self.refuse_if_exceeding_total(
            project.id,
            quote.net_cents,
            amount_cents,
            command.allow_exceeding_total,
        )
        .await?;

        let label = format!("Acompte ({})", format_percentage_bp(command.percentage_bp));
        let issued = self
            .issue_single_line(
                project.organization_id,
                SingleLineDraft {
                    invoice_id: InvoiceId(generate_uuid_v7()),
                    organization_id: project.organization_id,
                    kind: InvoiceKind::Deposit,
                    project_id: project.id,
                    customer_id,
                    customer_context_id,
                    notes: command.notes,
                    label,
                    amount_cents,
                    payment_terms_days: command.payment_terms_days,
                    now,
                },
                command.allow_exceeding_total,
                organizations,
                projects,
                quotes,
                customers,
                contexts,
            )
            .await;

        settle(issued)
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn issue_final_invoice<O, P, Q, C, X>(
        &mut self,
        command: IssueFinalInvoiceCommand,
        organizations: &mut O,
        projects: &mut P,
        quotes: &mut Q,
        customers: &mut C,
        contexts: &mut X,
        now: DateTime<Utc>,
    ) -> Result<InvoiceOutcome<Invoice>, CoreError>
    where
        O: OrganizationRepository,
        P: ProjectRepository,
        Q: QuoteRepository,
        C: CustomerRepository,
        X: CustomerContextRepository,
    {
        let project = projects
            .find_by_id(command.project_id)
            .await?
            .ok_or(CoreError::NotFound)?;
        self.require_manage_invoices(command.actor.clone(), project.organization_id)
            .await?;
        let quote_id = project.quote_id.ok_or_else(|| {
            CoreError::Conflict(format!(
                "project {} has no quote; a final invoice needs a total to compute the remainder against",
                project.id
            ))
        })?;
        let (customer_id, customer_context_id) = required_customer_fields(&project)?;
        let quote = quotes
            .find_by_id(quote_id)
            .await?
            .ok_or(CoreError::NotFound)?;

        let already_issued = self.sum_already_issued_cents(project.id).await?;
        let remaining = remaining_to_bill_cents(quote.net_cents, already_issued)?;
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

        let issued = self
            .issue_single_line(
                project.organization_id,
                SingleLineDraft {
                    invoice_id: InvoiceId(generate_uuid_v7()),
                    organization_id: project.organization_id,
                    kind: InvoiceKind::Final,
                    project_id: project.id,
                    customer_id,
                    customer_context_id,
                    notes: command.notes,
                    label: "Solde".to_owned(),
                    amount_cents,
                    payment_terms_days: command.payment_terms_days,
                    now,
                },
                command.allow_exceeding_total,
                organizations,
                projects,
                quotes,
                customers,
                contexts,
            )
            .await;

        settle(issued)
    }

    #[allow(clippy::too_many_arguments)]
    async fn issue_single_line<O, P, Q, C, X>(
        &mut self,
        organization_id: OrganizationId,
        single: SingleLineDraft,
        allow_exceeding_total: bool,
        organizations: &mut O,
        projects: &mut P,
        quotes: &mut Q,
        customers: &mut C,
        contexts: &mut X,
    ) -> Result<Invoice, Stop>
    where
        O: OrganizationRepository,
        P: ProjectRepository,
        Q: QuoteRepository,
        C: CustomerRepository,
        X: CustomerContextRepository,
    {
        let customer = require_customer(customers, organization_id, single.customer_id).await?;
        let settings = organizations
            .find_billing_settings(organization_id)
            .await?
            .unwrap_or_default();
        let terms = draft_terms(single.payment_terms_days, None, &customer, &settings)?;

        let now = single.now;
        let today = now.date_naive();
        let draft = build_single_line_draft(single, terms);
        let ready = self
            .check_issuable(
                draft.invoice(),
                today,
                today,
                organizations,
                customers,
                contexts,
            )
            .await?;

        let inserted = self.repo.insert_draft(&draft).await?;
        self.emitter.emit(
            inserted.organization_id,
            &InvoiceCreated {
                invoice: inserted.clone(),
            },
        )?;

        self.perform_issue(
            inserted,
            ready,
            today,
            allow_exceeding_total,
            projects,
            quotes,
            now,
        )
        .await
    }

    pub async fn issue_credit_note<O>(
        &mut self,
        command: IssueCreditNoteCommand,
        organizations: &mut O,
        now: DateTime<Utc>,
    ) -> Result<InvoiceOutcome<Invoice>, CoreError>
    where
        O: OrganizationRepository,
    {
        let source = self.load(command.source_invoice_id).await?;
        self.require_manage_invoices(command.actor.clone(), source.organization_id)
            .await?;

        settle(self.credit(source, command, organizations, now).await)
    }

    async fn credit<O>(
        &mut self,
        source: Invoice,
        command: IssueCreditNoteCommand,
        organizations: &mut O,
        now: DateTime<Utc>,
    ) -> Result<Invoice, Stop>
    where
        O: OrganizationRepository,
    {
        if source.kind == InvoiceKind::CreditNote {
            return Err(Stop::Core(CoreError::Conflict(format!(
                "invoice {} is itself a credit note and cannot be credited",
                source.id
            ))));
        }
        if source.status != InvoiceStatus::Issued {
            return Err(InvoiceError::NotIssued.into());
        }
        let reason = command.reason.trim().to_owned();
        if reason.is_empty() {
            return Err(InvoiceError::invalid(
                InvoiceField::Reason,
                "the reason of a credit note is required",
            )
            .into());
        }

        let today = now.date_naive();
        let invoice_id = InvoiceId(generate_uuid_v7());
        let credit_notes = self.repo.list_by_source_invoice(source.id).await?;
        let lines = credit_note_lines(&source, &credit_notes, &command.request, invoice_id, now)?;
        let last_credit_note_on = self
            .repo
            .last_credit_note_on(source.organization_id)
            .await?;
        check_credit_note_date(today, last_credit_note_on, source.issued_on)?;

        let organization = resolve_organization(organizations, source.organization_id).await?;
        let settings = organizations
            .find_billing_settings(source.organization_id)
            .await?
            .unwrap_or_default();
        let pricing = totals_of(&lines, is_subject_to_vat(&organization))?;
        if pricing.gross_cents < 0 {
            return Err(InvoiceError::invalid(
                InvoiceField::Total,
                "a credit note total cannot be negative",
            )
            .into());
        }
        let issuer = source.issuer_identity.clone().ok_or_else(|| {
            CoreError::Internal(format!(
                "issued invoice {} has no issuer identity",
                source.id
            ))
        })?;
        let full_cancellation = fully_credits(&source, &credit_notes, &lines);

        let draft = DraftInvoice::try_from_invoice(Invoice {
            id: invoice_id,
            organization_id: source.organization_id,
            number: None,
            kind: InvoiceKind::CreditNote,
            project_id: source.project_id,
            customer_id: source.customer_id,
            customer_context_id: source.customer_context_id,
            status: InvoiceStatus::Draft,
            issued_at: None,
            due_at: None,
            notes: normalized(command.notes),
            operation_nature: None,
            delivery_address: None,
            net_cents: pricing.net_cents,
            vat_breakdown: pricing.vat_breakdown,
            gross_cents: pricing.gross_cents,
            issuer_identity: None,
            generated_document: None,
            lines,
            source_invoice_id: Some(source.id),
            title: source.title.clone(),
            payment_terms_days: source.payment_terms_days,
            legal_notice: String::new(),
            internal_note: None,
            sent_at: None,
            quote_id: None,
            issued_on: None,
            reason: Some(reason),
            full_cancellation,
            derived: InvoiceDerived::default(),
            deleted_at: None,
            created_at: now,
            updated_at: now,
        })
        .expect("just constructed with status Draft");

        let inserted = self.repo.insert_draft(&draft).await?;
        self.emitter.emit(
            inserted.organization_id,
            &InvoiceCreated {
                invoice: inserted.clone(),
            },
        )?;

        let number = self
            .repo
            .allocate_number(
                inserted.organization_id,
                InvoiceNumberSeries::CreditNote,
                CREDIT_NOTE_NUMBER_PREFIX,
                today.year(),
                settings.number_padding,
            )
            .await?;
        let issued = self
            .repo
            .issue(inserted.id, number, now, today, None, &issuer, now)
            .await?;

        self.emit_transition(InvoiceStatus::Draft, &issued)?;

        Ok(decorate(issued, None, today))
    }

    pub async fn create_invoice_from_quote<O, Q, C>(
        &mut self,
        command: CreateInvoiceFromQuoteCommand,
        organizations: &mut O,
        quotes: &mut Q,
        customers: &mut C,
        now: DateTime<Utc>,
    ) -> Result<InvoiceOutcome<Invoice>, CoreError>
    where
        O: OrganizationRepository,
        Q: QuoteRepository,
        C: CustomerRepository,
    {
        let quote = quotes
            .find_by_id(command.quote_id)
            .await?
            .ok_or(CoreError::NotFound)?;
        self.require_manage_invoices(command.actor.clone(), quote.organization_id)
            .await?;

        settle(
            self.invoice_from_quote(quote, command.mode, organizations, customers, now)
                .await,
        )
    }

    async fn invoice_from_quote<O, C>(
        &mut self,
        quote: Quote,
        mode: QuoteInvoiceMode,
        organizations: &mut O,
        customers: &mut C,
        now: DateTime<Utc>,
    ) -> Result<Invoice, Stop>
    where
        O: OrganizationRepository,
        C: CustomerRepository,
    {
        let today = now.date_naive();
        let active = self.active_quote_invoices(quote.id, today).await?;
        let info = quote_invoicing(&quote, &active);
        match info.blocker {
            Some(rules::QuoteInvoicingBlocker::QuoteNotAccepted) => {
                return Err(InvoiceError::QuoteNotAccepted.into());
            }
            Some(rules::QuoteInvoicingBlocker::QuoteAlreadyInvoiced) => {
                return Err(InvoiceError::QuoteAlreadyInvoiced.into());
            }
            Some(rules::QuoteInvoicingBlocker::DepositsCoverQuote) | None => {}
        }

        let invoice_id = InvoiceId(generate_uuid_v7());
        let target = DocumentLines {
            organization_id: quote.organization_id,
            invoice_id,
            now,
        };
        let (kind, lines) = match mode {
            QuoteInvoiceMode::Full => {
                let issued_deposits: Vec<Invoice> = active
                    .iter()
                    .filter(|invoice| {
                        invoice.kind == InvoiceKind::Deposit
                            && invoice.status != InvoiceStatus::Draft
                    })
                    .cloned()
                    .collect();
                (
                    InvoiceKind::Standard,
                    full_invoice_lines(&quote, &issued_deposits, &target)?,
                )
            }
            QuoteInvoiceMode::Deposit { percent } => {
                let percent = DepositPercent::try_from(percent).map_err(|_| {
                    InvoiceError::invalid(
                        InvoiceField::Percent,
                        "must be between 0 and 100 with at most one decimal",
                    )
                })?;
                if percent.get() <= Decimal::ZERO {
                    return Err(InvoiceError::invalid(
                        InvoiceField::Percent,
                        "must be greater than zero",
                    )
                    .into());
                }
                if !info.can_deposit || percent.get() + info.invoiced_percent > Decimal::ONE_HUNDRED
                {
                    return Err(InvoiceError::DepositExceedsRemaining.into());
                }
                (
                    InvoiceKind::Deposit,
                    deposit_lines(&quote, percent, &target)?,
                )
            }
        };

        let organization = resolve_organization(organizations, quote.organization_id).await?;
        let customer =
            require_customer(customers, quote.organization_id, quote.customer_id).await?;
        let settings = organizations
            .find_billing_settings(quote.organization_id)
            .await?
            .unwrap_or_default();
        let terms = draft_terms(None, None, &customer, &settings)?;
        let pricing = totals_of(&lines, is_subject_to_vat(&organization))?;

        let draft = DraftInvoice::try_from_invoice(Invoice {
            id: invoice_id,
            organization_id: quote.organization_id,
            number: None,
            kind,
            project_id: None,
            customer_id: quote.customer_id,
            customer_context_id: quote
                .billing_address_id
                .or(customer.profile.billing_address_id)
                .unwrap_or(quote.customer_context_id),
            status: InvoiceStatus::Draft,
            issued_at: None,
            due_at: None,
            notes: None,
            operation_nature: None,
            delivery_address: None,
            net_cents: pricing.net_cents,
            vat_breakdown: pricing.vat_breakdown,
            gross_cents: pricing.gross_cents,
            issuer_identity: None,
            generated_document: None,
            lines,
            source_invoice_id: None,
            title: quote.title.trim().to_owned(),
            payment_terms_days: terms.payment_terms_days,
            legal_notice: terms.legal_notice,
            internal_note: None,
            sent_at: None,
            quote_id: Some(quote.id),
            issued_on: None,
            reason: None,
            full_cancellation: false,
            derived: InvoiceDerived::default(),
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

        Ok(decorate(created, None, today))
    }

    async fn active_quote_invoices(
        &mut self,
        quote_id: QuoteId,
        today: NaiveDate,
    ) -> Result<Vec<Invoice>, CoreError> {
        let invoices = self.repo.list_by_quote(quote_id).await?;
        let decorated = self.decorate_many(invoices, today).await?;

        Ok(decorated.into_iter().filter(is_active_on_quote).collect())
    }

    pub async fn quote_invoicing<Q>(
        &mut self,
        quote_id: QuoteId,
        quotes: &mut Q,
        today: NaiveDate,
    ) -> Result<QuoteInvoicingView, CoreError>
    where
        Q: QuoteRepository,
    {
        let quote = quotes
            .find_by_id(quote_id)
            .await?
            .ok_or(CoreError::NotFound)?;
        let active = self.active_quote_invoices(quote_id, today).await?;

        Ok(QuoteInvoicingView {
            organization_id: quote.organization_id,
            invoicing: quote_invoicing(&quote, &active),
            invoices: active,
        })
    }

    pub async fn send_invoice(
        &mut self,
        command: SendInvoiceCommand,
        now: DateTime<Utc>,
    ) -> Result<InvoiceOutcome<Invoice>, CoreError> {
        let invoice = self.load(command.id).await?;
        self.require_manage_invoices(command.actor, invoice.organization_id)
            .await?;
        if invoice.status != InvoiceStatus::Issued {
            return Ok(Err(InvoiceError::NotIssued));
        }

        self.repo.mark_sent(invoice.id, now).await?;
        self.record_event(
            &invoice,
            InvoiceEventKind::Sent,
            "Envoyée au client".to_owned(),
            command.recorded_by,
            now,
        )
        .await?;

        Ok(Ok(self.get_invoice(invoice.id, now.date_naive()).await?))
    }

    pub async fn remind_invoice(
        &mut self,
        command: RemindInvoiceCommand,
        now: DateTime<Utc>,
    ) -> Result<InvoiceOutcome<Invoice>, CoreError> {
        let invoice = self.load(command.id).await?;
        self.require_manage_invoices(command.actor, invoice.organization_id)
            .await?;
        if invoice.status != InvoiceStatus::Issued {
            return Ok(Err(InvoiceError::NotIssued));
        }

        let detail = reminder_detail(invoice.due_on(), now.date_naive());
        self.record_event(
            &invoice,
            InvoiceEventKind::Reminder,
            detail,
            command.recorded_by,
            now,
        )
        .await?;

        Ok(Ok(self.get_invoice(invoice.id, now.date_naive()).await?))
    }

    pub async fn note_invoice(
        &mut self,
        command: NoteInvoiceCommand,
        now: DateTime<Utc>,
    ) -> Result<InvoiceOutcome<Invoice>, CoreError> {
        let invoice = self.load(command.id).await?;
        self.require_manage_invoices(command.actor, invoice.organization_id)
            .await?;

        let note = command.note.trim().to_owned();
        self.repo.set_internal_note(invoice.id, &note, now).await?;
        if !note.is_empty() {
            self.record_event(
                &invoice,
                InvoiceEventKind::Note,
                note,
                command.recorded_by,
                now,
            )
            .await?;
        }

        Ok(Ok(self.get_invoice(invoice.id, now.date_naive()).await?))
    }

    async fn record_event(
        &mut self,
        invoice: &Invoice,
        kind: InvoiceEventKind,
        detail: String,
        actor: UserId,
        at: DateTime<Utc>,
    ) -> Result<InvoiceEvent, CoreError> {
        self.repo
            .insert_event(
                invoice.organization_id,
                invoice.id,
                &InvoiceEvent {
                    id: InvoiceEventId(generate_uuid_v7()),
                    kind,
                    detail,
                    at,
                    actor: Some(actor),
                },
            )
            .await
    }

    async fn check_issuable<O, C, X>(
        &mut self,
        invoice: &Invoice,
        issue_date: NaiveDate,
        today: NaiveDate,
        organizations: &mut O,
        customers: &mut C,
        contexts: &mut X,
    ) -> Result<Issuable, Stop>
    where
        O: OrganizationRepository,
        C: CustomerRepository,
        X: CustomerContextRepository,
    {
        let customer =
            require_customer(customers, invoice.organization_id, invoice.customer_id).await?;
        let address = contexts
            .find_by_id(invoice.customer_context_id)
            .await?
            .filter(|address| address.customer_id == customer.id);
        let recipient = RecipientSnapshot::of(&customer, None, address.as_ref());
        let last_issued_on = self.repo.last_issued_on(invoice.organization_id).await?;

        validate_issue(&IssueFacts {
            invoice,
            customer_kind: customer.profile.kind,
            registration_number: customer.registration_number.as_deref(),
            billing_address_is_postal: recipient.address.is_postal(),
            issue_date,
            today,
            last_issued_on,
        })?;

        let organization = resolve_organization(organizations, invoice.organization_id).await?;
        let legal_identity =
            LegalIdentity::try_from_organization(&organization).map_err(|missing_fields| {
                CoreError::Conflict(format!(
                    "cannot issue invoice {}: the organization's legal identity is missing {}",
                    invoice.id,
                    missing_fields.join(", ")
                ))
            })?;
        let settings = organizations
            .find_billing_settings(invoice.organization_id)
            .await?
            .unwrap_or_default();

        Ok(Issuable {
            organization,
            legal_identity: legal_identity.with_billing(IssuerBilling::from(&settings)),
            settings,
        })
    }

    #[allow(clippy::too_many_arguments)]
    async fn perform_issue<P, Q>(
        &mut self,
        invoice: Invoice,
        ready: Issuable,
        issue_date: NaiveDate,
        allow_exceeding_total: bool,
        projects: &mut P,
        quotes: &mut Q,
        now: DateTime<Utc>,
    ) -> Result<Invoice, Stop>
    where
        P: ProjectRepository,
        Q: QuoteRepository,
    {
        self.refuse_if_project_exceeds_total(&invoice, allow_exceeding_total, projects, quotes)
            .await?;

        let series = InvoiceNumberSeries::for_kind(invoice.kind);
        let prefix = match series {
            InvoiceNumberSeries::Invoice => ready.organization.invoice_number_prefix.as_str(),
            InvoiceNumberSeries::CreditNote => CREDIT_NOTE_NUMBER_PREFIX,
        };
        let number = self
            .repo
            .allocate_number(
                invoice.organization_id,
                series,
                prefix,
                issue_date.year(),
                ready.settings.number_padding,
            )
            .await?;

        let issued = self
            .repo
            .issue(
                invoice.id,
                number,
                now,
                issue_date,
                Some(due_at_for(issue_date, invoice.payment_terms_days)),
                &ready.legal_identity,
                now,
            )
            .await?;

        self.emit_transition(InvoiceStatus::Draft, &issued)?;

        Ok(decorate(issued, None, now.date_naive()))
    }

    async fn refuse_if_project_exceeds_total(
        &mut self,
        invoice: &Invoice,
        allow_exceeding_total: bool,
        project_repository: &mut impl ProjectRepository,
        quote_repository: &mut impl QuoteRepository,
    ) -> Result<(), CoreError> {
        if invoice.kind == InvoiceKind::CreditNote {
            return Ok(());
        }

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

        self.refuse_if_exceeding_total(
            project_id,
            quote.net_cents,
            invoice.net_cents,
            allow_exceeding_total,
        )
        .await
    }

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
            .map(|invoice| match invoice.kind {
                InvoiceKind::CreditNote => -i64::from(invoice.net_cents),
                InvoiceKind::Standard | InvoiceKind::Deposit | InvoiceKind::Final => {
                    i64::from(invoice.net_cents)
                }
            })
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

fn decorate(
    mut invoice: Invoice,
    collection: Option<&InvoiceCollection>,
    today: NaiveDate,
) -> Invoice {
    let persisted = invoice.status;
    let settlement = settlement_of_collection(&invoice, collection);

    invoice.derived.display_state = display_state(
        persisted,
        invoice.kind,
        invoice.gross_cents,
        invoice.due_on(),
        settlement,
        today,
    );
    invoice.derived.settlement = settlement;
    invoice.status = derive_invoice_status(persisted, invoice.kind, settlement);

    invoice
}

fn settlement_of_collection(
    invoice: &Invoice,
    collection: Option<&InvoiceCollection>,
) -> InvoiceSettlement {
    let (paid, credited) = collection.map_or((0, 0), |collection| {
        (collection.paid_cents, collection.credited_cents)
    });

    settlement_of(
        invoice.status,
        invoice.kind,
        invoice.gross_cents,
        credited,
        paid,
    )
}

pub fn derive_invoice_status(
    persisted_status: InvoiceStatus,
    kind: InvoiceKind,
    settlement: InvoiceSettlement,
) -> InvoiceStatus {
    if persisted_status != InvoiceStatus::Issued || kind == InvoiceKind::CreditNote {
        return persisted_status;
    }

    if settlement.paid_cents > 0 && settlement.balance_cents <= 0 {
        InvoiceStatus::Paid
    } else if settlement.paid_cents > 0 {
        InvoiceStatus::PartiallyPaid
    } else {
        InvoiceStatus::Issued
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

async fn require_customer(
    customers: &mut impl CustomerRepository,
    organization_id: OrganizationId,
    customer_id: CustomerId,
) -> Result<Customer, CoreError> {
    customers
        .find_by_id(customer_id)
        .await?
        .filter(|customer| customer.organization_id == organization_id)
        .ok_or(CoreError::NotFound)
}

fn is_subject_to_vat(organization: &Organization) -> bool {
    matches!(organization.vat_status, Some(VatStatus::Subject { .. }))
}

fn normalized(text: Option<String>) -> Option<String> {
    text.map(|text| text.trim().to_owned())
        .filter(|text| !text.is_empty())
}

fn payment_terms_of(days: i64) -> InvoiceOutcome<i32> {
    PaymentTermsDays::try_from(days)
        .map(PaymentTermsDays::get)
        .map_err(|_| {
            InvoiceError::invalid(
                InvoiceField::PaymentTermsDays,
                "must be between 0 and 60 days",
            )
        })
}

fn draft_terms(
    payment_terms_days: Option<i64>,
    legal_notice: Option<String>,
    customer: &Customer,
    settings: &BillingSettings,
) -> InvoiceOutcome<DraftTerms> {
    let payment_terms_days = match payment_terms_days {
        Some(days) => payment_terms_of(days)?,
        None => customer
            .profile
            .payment_terms
            .map_or(settings.payment_terms_days.get(), PaymentTermsDays::get),
    };

    Ok(DraftTerms {
        payment_terms_days,
        legal_notice: normalized(legal_notice)
            .unwrap_or_else(|| legal_notice_for(customer.profile.kind, settings)),
    })
}

fn content_diff(existing: &Invoice, updated: &Invoice) -> (Vec<&'static str>, Value) {
    let mut changed = Vec::new();
    let mut previous = serde_json::Map::new();
    let mut moved = |field: &'static str, was: Value, differs: bool| {
        if differs {
            changed.push(field);
            previous.insert(field.to_owned(), was);
        }
    };

    moved(
        "project_id",
        json!(existing.project_id.map(|id| id.0)),
        existing.project_id != updated.project_id,
    );
    moved(
        "customer_id",
        json!(existing.customer_id.0),
        existing.customer_id != updated.customer_id,
    );
    moved(
        "customer_context_id",
        json!(existing.customer_context_id.0),
        existing.customer_context_id != updated.customer_context_id,
    );
    moved(
        "title",
        json!(existing.title),
        existing.title != updated.title,
    );
    moved(
        "payment_terms_days",
        json!(existing.payment_terms_days),
        existing.payment_terms_days != updated.payment_terms_days,
    );
    moved(
        "legal_notice",
        json!(existing.legal_notice),
        existing.legal_notice != updated.legal_notice,
    );
    moved(
        "notes",
        json!(existing.notes),
        existing.notes != updated.notes,
    );
    moved(
        "operation_nature",
        json!(existing.operation_nature.map(|n| n.as_str())),
        existing.operation_nature != updated.operation_nature,
    );
    moved(
        "delivery_address",
        json!(existing.delivery_address.is_some()),
        existing.delivery_address != updated.delivery_address,
    );
    let lines_moved = line_projection(existing) != line_projection(updated);
    moved("lines", line_projection(existing), lines_moved);
    if lines_moved {
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
                "discount_percent": line.discount_percent.get().normalize().to_string(),
                "product_id": line.product_id.map(|id| id.0),
                "service_rate_id": line.service_rate_id.map(|id| id.0),
                "position": line.position,
            }))
            .collect::<Vec<_>>()
    )
}

fn build_invoice_lines(
    organization_id: OrganizationId,
    invoice_id: InvoiceId,
    commands: Vec<InvoiceLineCommand>,
    now: DateTime<Utc>,
) -> InvoiceOutcome<Vec<InvoiceLine>> {
    if commands.is_empty() {
        return Err(InvoiceError::invalid(
            InvoiceField::Lines,
            "add at least one line",
        ));
    }

    commands
        .into_iter()
        .enumerate()
        .map(|(position, command)| {
            if command.product_id.is_some() && command.service_rate_id.is_some() {
                return Err(InvoiceError::invalid(
                    InvoiceField::Lines,
                    "a line references one catalogue item at most",
                ));
            }
            let line = InvoiceLine {
                id: InvoiceLineId(generate_uuid_v7()),
                organization_id,
                invoice_id,
                label: command.label.trim().to_owned(),
                quantity: command.quantity,
                unit_price_cents: command.unit_price_cents,
                vat_rate_basis_points: command.vat_rate_basis_points,
                discount_percent: discount_of(command.discount_percent)?,
                product_id: command.product_id,
                service_rate_id: command.service_rate_id,
                source_line_id: None,
                position: i32::try_from(position).unwrap_or(i32::MAX),
                deleted_at: None,
                created_at: now,
                updated_at: now,
            };
            validate_line(&line)?;

            Ok(line)
        })
        .collect()
}

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

fn deposit_amount_cents(percentage_bp: i32, quoted_cents: i32) -> Result<i32, CoreError> {
    let amount = div_round_half_even(i64::from(percentage_bp) * i64::from(quoted_cents), 10_000);

    i32::try_from(amount)
        .map_err(|_| CoreError::Conflict("deposit amount is outside supported bounds".to_owned()))
}

fn format_percentage_bp(percentage_bp: i32) -> String {
    let whole = percentage_bp / 100;
    let hundredths = percentage_bp % 100;

    if hundredths == 0 {
        format!("{whole}%")
    } else {
        format!("{whole}.{hundredths:02}%")
    }
}

fn build_single_line_draft(single: SingleLineDraft, terms: DraftTerms) -> DraftInvoice {
    let line = InvoiceLine {
        id: InvoiceLineId(generate_uuid_v7()),
        organization_id: single.organization_id,
        invoice_id: single.invoice_id,
        label: single.label,
        quantity: Decimal::ONE,
        unit_price_cents: single.amount_cents,
        vat_rate_basis_points: None,
        discount_percent: crate::DiscountPercent::ZERO,
        product_id: None,
        service_rate_id: None,
        source_line_id: None,
        position: 0,
        deleted_at: None,
        created_at: single.now,
        updated_at: single.now,
    };

    DraftInvoice::try_from_invoice(Invoice {
        id: single.invoice_id,
        organization_id: single.organization_id,
        number: None,
        kind: single.kind,
        project_id: Some(single.project_id),
        customer_id: single.customer_id,
        customer_context_id: single.customer_context_id,
        status: InvoiceStatus::Draft,
        issued_at: None,
        due_at: None,
        notes: single.notes,
        operation_nature: None,
        delivery_address: None,
        net_cents: single.amount_cents,
        vat_breakdown: Vec::new(),
        gross_cents: single.amount_cents,
        issuer_identity: None,
        generated_document: None,
        lines: vec![line],
        source_invoice_id: None,
        title: String::new(),
        payment_terms_days: terms.payment_terms_days,
        legal_notice: terms.legal_notice,
        internal_note: None,
        sent_at: None,
        quote_id: None,
        issued_on: None,
        reason: None,
        full_cancellation: false,
        derived: InvoiceDerived::default(),
        deleted_at: None,
        created_at: single.now,
        updated_at: single.now,
    })
    .expect("just constructed with status Draft")
}

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
#[path = "service_tests.rs"]
mod tests;
