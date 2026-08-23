use common::CoreError;
use mestier_macros::transactional;

use crate::{
    Invoice, InvoiceId, OrganizationId, ProjectId,
    application::MestierUseCase,
    domain::invoice::{
        commands::{
            CancelInvoiceCommand, CreateInvoiceCommand, IssueCreditNoteCommand,
            IssueDepositCommand, IssueFinalInvoiceCommand, IssueInvoiceCommand,
            UpdateInvoiceCommand,
        },
        service::InvoiceService,
    },
};

impl MestierUseCase {
    /// `organization` reads the organization's VAT status, same reason as
    /// `create_quote`: totals depend on it, and only calls that can
    /// recompute totals need to be aware of the organization aggregate.
    #[transactional(invoice, organization, emitter)]
    pub async fn create_invoice(
        &self,
        command: CreateInvoiceCommand,
    ) -> Result<Invoice, CoreError> {
        let mut service = InvoiceService::new(invoice_repository, emitter);
        service
            .create_invoice(command, organization_repository)
            .await
    }

    #[transactional(invoice, emitter)]
    pub async fn get_invoice(&self, id: InvoiceId) -> Result<Invoice, CoreError> {
        let mut service = InvoiceService::new(invoice_repository, emitter);
        service.get_invoice(id).await
    }

    #[transactional(invoice, emitter)]
    pub async fn list_invoices(
        &self,
        organization_id: OrganizationId,
        limit: u64,
        offset: u64,
    ) -> Result<(Vec<Invoice>, u64), CoreError> {
        let mut service = InvoiceService::new(invoice_repository, emitter);
        service.list_invoices(organization_id, limit, offset).await
    }

    #[transactional(invoice, emitter)]
    pub async fn list_invoices_by_project(
        &self,
        project_id: ProjectId,
    ) -> Result<Vec<Invoice>, CoreError> {
        let mut service = InvoiceService::new(invoice_repository, emitter);
        service.list_invoices_by_project(project_id).await
    }

    #[transactional(invoice, emitter)]
    pub async fn get_invoice_credit_notes(
        &self,
        source_invoice_id: InvoiceId,
    ) -> Result<Vec<Invoice>, CoreError> {
        let mut service = InvoiceService::new(invoice_repository, emitter);
        service.get_invoice_credit_notes(source_invoice_id).await
    }

    #[transactional(invoice, organization, emitter)]
    pub async fn update_invoice(
        &self,
        command: UpdateInvoiceCommand,
    ) -> Result<Invoice, CoreError> {
        let mut service = InvoiceService::new(invoice_repository, emitter);
        service
            .update_invoice(command, organization_repository)
            .await
    }

    #[transactional(invoice, emitter)]
    pub async fn cancel_invoice(
        &self,
        command: CancelInvoiceCommand,
    ) -> Result<Invoice, CoreError> {
        let mut service = InvoiceService::new(invoice_repository, emitter);
        service.cancel_invoice(command).await
    }

    #[transactional(invoice, emitter)]
    pub async fn soft_delete_invoice(&self, id: InvoiceId) -> Result<(), CoreError> {
        let mut service = InvoiceService::new(invoice_repository, emitter);
        service.soft_delete_invoice(id).await
    }

    /// `project` and `quote` resolve the invoice's project and that
    /// project's quote, when it has one — the reads `issue_invoice` needs to
    /// refuse an over-issuance, and `organization` resolves the issuer's
    /// legal identity and invoice number prefix, same reason `create_invoice`
    /// lists `organization`.
    #[transactional(invoice, organization, project, quote, emitter)]
    pub async fn issue_invoice(&self, command: IssueInvoiceCommand) -> Result<Invoice, CoreError> {
        let mut service = InvoiceService::new(invoice_repository, emitter);
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
    #[transactional(invoice, organization, project, quote, emitter)]
    pub async fn issue_deposit(&self, command: IssueDepositCommand) -> Result<Invoice, CoreError> {
        let mut service = InvoiceService::new(invoice_repository, emitter);
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
    #[transactional(invoice, organization, project, quote, emitter)]
    pub async fn issue_final_invoice(
        &self,
        command: IssueFinalInvoiceCommand,
    ) -> Result<Invoice, CoreError> {
        let mut service = InvoiceService::new(invoice_repository, emitter);
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
    #[transactional(invoice, organization, project, quote, emitter)]
    pub async fn issue_credit_note(
        &self,
        command: IssueCreditNoteCommand,
    ) -> Result<Invoice, CoreError> {
        let mut service = InvoiceService::new(invoice_repository, emitter);
        service
            .issue_credit_note(
                command,
                organization_repository,
                project_repository,
                quote_repository,
            )
            .await
    }
}
