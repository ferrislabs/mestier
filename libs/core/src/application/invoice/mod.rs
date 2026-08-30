use authz::Subject;
use chrono::{DateTime, Utc};
use common::CoreError;
use mestier_macros::transactional;

use crate::{
    CustomerOutstandingBalance, GeneratedInvoiceDocument, Invoice, InvoiceId, InvoicePayment,
    InvoicePaymentId, OrganizationId, ProjectBillingSummary, ProjectId,
    application::MestierUseCase,
    domain::invoice::{
        commands::{
            CancelInvoiceCommand, CreateInvoiceCommand, DeleteInvoicePaymentCommand,
            IssueCreditNoteCommand, IssueDepositCommand, IssueFinalInvoiceCommand,
            IssueInvoiceCommand, RecordInvoicePaymentCommand, UpdateInvoiceCommand,
        },
        service::InvoiceService,
    },
};

impl MestierUseCase {
    /// `organization` reads the organization's VAT status, same reason as
    /// `create_quote`: totals depend on it, and only calls that can
    /// recompute totals need to be aware of the organization aggregate.
    /// `role`/`member`/`authz` (#395) let `InvoiceService` enforce
    /// `MANAGE_INVOICES` before the write reaches the repository — the same
    /// trio `create_customer` lists for `customer.manage`.
    #[transactional(invoice, organization, role, member, authz, emitter)]
    pub async fn create_invoice(
        &self,
        command: CreateInvoiceCommand,
    ) -> Result<Invoice, CoreError> {
        let mut service = InvoiceService::new(
            invoice_repository,
            emitter,
            member_repository,
            role_repository,
            authz,
        );
        service
            .create_invoice(command, organization_repository)
            .await
    }

    /// `role`/`member`/`authz` are listed even though this read never calls
    /// `policy::require` (same reason `CustomerService::get_customer` lists
    /// them): `InvoiceService` is generic over all three, so constructing it
    /// needs concrete bindings regardless of which method runs.
    #[transactional(invoice, role, member, authz, emitter)]
    pub async fn get_invoice(&self, id: InvoiceId) -> Result<Invoice, CoreError> {
        let mut service = InvoiceService::new(
            invoice_repository,
            emitter,
            member_repository,
            role_repository,
            authz,
        );
        service.get_invoice(id).await
    }

    #[transactional(invoice, role, member, authz, emitter)]
    pub async fn list_invoices(
        &self,
        organization_id: OrganizationId,
        limit: u64,
        offset: u64,
    ) -> Result<(Vec<Invoice>, u64), CoreError> {
        let mut service = InvoiceService::new(
            invoice_repository,
            emitter,
            member_repository,
            role_repository,
            authz,
        );
        service.list_invoices(organization_id, limit, offset).await
    }

    #[transactional(invoice, role, member, authz, emitter)]
    pub async fn list_invoices_by_project(
        &self,
        project_id: ProjectId,
    ) -> Result<Vec<Invoice>, CoreError> {
        let mut service = InvoiceService::new(
            invoice_repository,
            emitter,
            member_repository,
            role_repository,
            authz,
        );
        service.list_invoices_by_project(project_id).await
    }

    #[transactional(invoice, role, member, authz, emitter)]
    pub async fn get_invoice_credit_notes(
        &self,
        source_invoice_id: InvoiceId,
    ) -> Result<Vec<Invoice>, CoreError> {
        let mut service = InvoiceService::new(
            invoice_repository,
            emitter,
            member_repository,
            role_repository,
            authz,
        );
        service.get_invoice_credit_notes(source_invoice_id).await
    }

    #[transactional(invoice, organization, role, member, authz, emitter)]
    pub async fn update_invoice(
        &self,
        command: UpdateInvoiceCommand,
    ) -> Result<Invoice, CoreError> {
        let mut service = InvoiceService::new(
            invoice_repository,
            emitter,
            member_repository,
            role_repository,
            authz,
        );
        service
            .update_invoice(command, organization_repository)
            .await
    }

    #[transactional(invoice, role, member, authz, emitter)]
    pub async fn cancel_invoice(
        &self,
        command: CancelInvoiceCommand,
    ) -> Result<Invoice, CoreError> {
        let mut service = InvoiceService::new(
            invoice_repository,
            emitter,
            member_repository,
            role_repository,
            authz,
        );
        service.cancel_invoice(command).await
    }

    #[transactional(invoice, role, member, authz, emitter)]
    pub async fn soft_delete_invoice(
        &self,
        id: InvoiceId,
        actor: Subject,
    ) -> Result<(), CoreError> {
        let mut service = InvoiceService::new(
            invoice_repository,
            emitter,
            member_repository,
            role_repository,
            authz,
        );
        service.soft_delete_invoice(id, actor).await
    }

    /// `project` and `quote` resolve the invoice's project and that
    /// project's quote, when it has one — the reads `issue_invoice` needs to
    /// refuse an over-issuance, and `organization` resolves the issuer's
    /// legal identity and invoice number prefix, same reason `create_invoice`
    /// lists `organization`. `role`/`member`/`authz` (#395) same reason as
    /// `create_invoice`.
    #[transactional(invoice, organization, project, quote, role, member, authz, emitter)]
    pub async fn issue_invoice(&self, command: IssueInvoiceCommand) -> Result<Invoice, CoreError> {
        let mut service = InvoiceService::new(
            invoice_repository,
            emitter,
            member_repository,
            role_repository,
            authz,
        );
        service
            .issue_invoice(
                command,
                organization_repository,
                project_repository,
                quote_repository,
            )
            .await
    }

    /// Builds a deposit draft and issues it, both inside this one
    /// transaction — the draft's number is allocated in the same
    /// transaction as its own creation, never in a second use-case call.
    #[transactional(invoice, organization, project, quote, role, member, authz, emitter)]
    pub async fn issue_deposit(&self, command: IssueDepositCommand) -> Result<Invoice, CoreError> {
        let mut service = InvoiceService::new(
            invoice_repository,
            emitter,
            member_repository,
            role_repository,
            authz,
        );
        service
            .issue_deposit(
                command,
                organization_repository,
                project_repository,
                quote_repository,
            )
            .await
    }

    /// Same one-transaction shape as `issue_deposit`, for the invoice that
    /// bills exactly what remains on the project's quote.
    #[transactional(invoice, organization, project, quote, role, member, authz, emitter)]
    pub async fn issue_final_invoice(
        &self,
        command: IssueFinalInvoiceCommand,
    ) -> Result<Invoice, CoreError> {
        let mut service = InvoiceService::new(
            invoice_repository,
            emitter,
            member_repository,
            role_repository,
            authz,
        );
        service
            .issue_final_invoice(
                command,
                organization_repository,
                project_repository,
                quote_repository,
            )
            .await
    }

    /// `project`/`quote` are listed even though a credit note's own path
    /// skips the quote-total check (see `InvoiceService::issue_credit_note`
    /// and `refuse_if_project_exceeds_total`'s credit-note skip) — the
    /// binding still needs to exist because `issue_now`, which this calls,
    /// is generic over `P: ProjectRepository, Q: QuoteRepository`
    /// regardless of which invoice kind is being issued.
    #[transactional(invoice, organization, project, quote, role, member, authz, emitter)]
    pub async fn issue_credit_note(
        &self,
        command: IssueCreditNoteCommand,
    ) -> Result<Invoice, CoreError> {
        let mut service = InvoiceService::new(
            invoice_repository,
            emitter,
            member_repository,
            role_repository,
            authz,
        );
        service
            .issue_credit_note(
                command,
                organization_repository,
                project_repository,
                quote_repository,
            )
            .await
    }

    /// #320: records a payment against an issued invoice. No other
    /// bindings needed — this reads nothing from `organization`/`project`/
    /// `quote`.
    #[transactional(invoice, role, member, authz, emitter)]
    pub async fn record_invoice_payment(
        &self,
        command: RecordInvoicePaymentCommand,
    ) -> Result<InvoicePayment, CoreError> {
        let mut service = InvoiceService::new(
            invoice_repository,
            emitter,
            member_repository,
            role_repository,
            authz,
        );
        service.record_payment(command).await
    }

    #[transactional(invoice, role, member, authz, emitter)]
    pub async fn delete_invoice_payment(
        &self,
        command: DeleteInvoicePaymentCommand,
    ) -> Result<(), CoreError> {
        let mut service = InvoiceService::new(
            invoice_repository,
            emitter,
            member_repository,
            role_repository,
            authz,
        );
        service.delete_payment(command).await
    }

    #[transactional(invoice, role, member, authz, emitter)]
    pub async fn list_invoice_payments(
        &self,
        invoice_id: InvoiceId,
    ) -> Result<Vec<InvoicePayment>, CoreError> {
        let mut service = InvoiceService::new(
            invoice_repository,
            emitter,
            member_repository,
            role_repository,
            authz,
        );
        service.list_payments(invoice_id).await
    }

    /// Not registered by #320: every one of its own use cases already holds
    /// the invoice, or a payment id plus an actor. #319's delete-by-
    /// payment-id HTTP route has only the payment id, and needs this to
    /// resolve the payment's own organization for the membership check
    /// before it can call `delete_invoice_payment` at all — the same
    /// "unavoidable one more file" #317/#318/#320 each flagged.
    #[transactional(invoice, role, member, authz, emitter)]
    pub async fn find_payment_by_id(
        &self,
        id: InvoicePaymentId,
    ) -> Result<Option<InvoicePayment>, CoreError> {
        let mut service = InvoiceService::new(
            invoice_repository,
            emitter,
            member_repository,
            role_repository,
            authz,
        );
        service.find_payment_by_id(id).await
    }

    #[transactional(invoice, role, member, authz, emitter)]
    pub async fn outstanding_balance_by_customer(
        &self,
        organization_id: OrganizationId,
    ) -> Result<Vec<CustomerOutstandingBalance>, CoreError> {
        let mut service = InvoiceService::new(
            invoice_repository,
            emitter,
            member_repository,
            role_repository,
            authz,
        );
        service.outstanding_by_customer(organization_id).await
    }

    #[transactional(invoice, role, member, authz, emitter)]
    pub async fn overdue_invoices(
        &self,
        organization_id: OrganizationId,
        as_of: DateTime<Utc>,
    ) -> Result<Vec<Invoice>, CoreError> {
        let mut service = InvoiceService::new(
            invoice_repository,
            emitter,
            member_repository,
            role_repository,
            authz,
        );
        service.overdue_invoices(organization_id, as_of).await
    }

    /// #342: persists the file a `DocumentFormat` adapter already generated
    /// (in the handler, against the already-uploaded bytes — see
    /// `InvoiceService::record_generated_document`'s own doc comment for
    /// why generation itself never happens inside this transaction).
    /// `role`/`member`/`authz` (#395): recording the generated document is a
    /// write on the invoice, gated by `MANAGE_INVOICES` like every other one.
    #[transactional(invoice, role, member, authz, emitter)]
    pub async fn record_invoice_generated_document(
        &self,
        id: InvoiceId,
        document: GeneratedInvoiceDocument,
        actor: Subject,
    ) -> Result<Invoice, CoreError> {
        let mut service = InvoiceService::new(
            invoice_repository,
            emitter,
            member_repository,
            role_repository,
            authz,
        );
        service.record_generated_document(id, document, actor).await
    }

    /// #319: "what was quoted, what has been billed, what remains" for one
    /// project — nothing in this module exposed it before, even though
    /// `InvoiceService`'s own `sum_already_issued_cents` did all the work
    /// privately. `project`/`quote` resolve the same two reads
    /// `issue_deposit`/`issue_final_invoice` already need for the same
    /// reason.
    #[transactional(invoice, project, quote, role, member, authz, emitter)]
    pub async fn project_billing_summary(
        &self,
        project_id: ProjectId,
    ) -> Result<ProjectBillingSummary, CoreError> {
        let mut service = InvoiceService::new(
            invoice_repository,
            emitter,
            member_repository,
            role_repository,
            authz,
        );
        service
            .project_billing_summary(project_id, project_repository, quote_repository)
            .await
    }
}
