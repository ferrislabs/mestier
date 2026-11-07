use authz::Subject;
use chrono::{DateTime, Utc};
use common::CoreError;
use mestier_macros::transactional;

use crate::{
    CustomerOutstandingBalance, GeneratedInvoiceDocument, Invoice, InvoiceId, InvoiceOutcome,
    InvoicePayment, InvoicePaymentId, OrganizationId, ProjectBillingSummary, ProjectId, QuoteId,
    application::MestierUseCase,
    domain::invoice::{
        commands::{
            CancelInvoiceCommand, CreateInvoiceCommand, CreateInvoiceFromQuoteCommand,
            DeleteInvoicePaymentCommand, IssueCreditNoteCommand, IssueDepositCommand,
            IssueFinalInvoiceCommand, IssueInvoiceCommand, NoteInvoiceCommand,
            RecordInvoicePaymentCommand, RemindInvoiceCommand, SendInvoiceCommand,
            UpdateInvoiceCommand,
        },
        service::{InvoiceService, QuoteInvoicingView},
    },
};

impl MestierUseCase {
    #[transactional(invoice, organization, customer, role, member, authz, emitter)]
    pub async fn create_invoice(
        &self,
        command: CreateInvoiceCommand,
    ) -> Result<InvoiceOutcome<Invoice>, CoreError> {
        let mut organization_repository = organization_repository;
        let mut customer_repository = customer_repository;
        let mut service = InvoiceService::new(
            invoice_repository,
            emitter,
            member_repository,
            role_repository,
            authz,
        );
        service
            .create_invoice(
                command,
                &mut organization_repository,
                &mut customer_repository,
                Utc::now(),
            )
            .await
    }

    #[transactional(invoice, role, member, authz, emitter)]
    pub async fn get_invoice(&self, id: InvoiceId) -> Result<Invoice, CoreError> {
        let mut service = InvoiceService::new(
            invoice_repository,
            emitter,
            member_repository,
            role_repository,
            authz,
        );
        service.get_invoice(id, Utc::now().date_naive()).await
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
        service
            .list_invoices(organization_id, limit, offset, Utc::now().date_naive())
            .await
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
        service
            .list_invoices_by_project(project_id, Utc::now().date_naive())
            .await
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
        service
            .get_invoice_credit_notes(source_invoice_id, Utc::now().date_naive())
            .await
    }

    #[transactional(invoice, organization, role, member, authz, emitter)]
    pub async fn update_invoice(
        &self,
        command: UpdateInvoiceCommand,
    ) -> Result<InvoiceOutcome<Invoice>, CoreError> {
        let mut organization_repository = organization_repository;
        let mut service = InvoiceService::new(
            invoice_repository,
            emitter,
            member_repository,
            role_repository,
            authz,
        );
        service
            .update_invoice(command, &mut organization_repository, Utc::now())
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
    ) -> Result<InvoiceOutcome<()>, CoreError> {
        let mut service = InvoiceService::new(
            invoice_repository,
            emitter,
            member_repository,
            role_repository,
            authz,
        );
        service.soft_delete_invoice(id, actor, Utc::now()).await
    }

    #[transactional(
        invoice,
        organization,
        project,
        quote,
        customer,
        customer_context,
        role,
        member,
        authz,
        emitter
    )]
    pub async fn issue_invoice(
        &self,
        command: IssueInvoiceCommand,
    ) -> Result<InvoiceOutcome<Invoice>, CoreError> {
        let mut organization_repository = organization_repository;
        let mut project_repository = project_repository;
        let mut quote_repository = quote_repository;
        let mut customer_repository = customer_repository;
        let mut customer_context_repository = customer_context_repository;
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
                &mut organization_repository,
                &mut project_repository,
                &mut quote_repository,
                &mut customer_repository,
                &mut customer_context_repository,
                Utc::now(),
            )
            .await
    }

    #[transactional(
        invoice,
        organization,
        project,
        quote,
        customer,
        customer_context,
        role,
        member,
        authz,
        emitter
    )]
    pub async fn issue_deposit(
        &self,
        command: IssueDepositCommand,
    ) -> Result<InvoiceOutcome<Invoice>, CoreError> {
        let mut organization_repository = organization_repository;
        let mut project_repository = project_repository;
        let mut quote_repository = quote_repository;
        let mut customer_repository = customer_repository;
        let mut customer_context_repository = customer_context_repository;
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
                &mut organization_repository,
                &mut project_repository,
                &mut quote_repository,
                &mut customer_repository,
                &mut customer_context_repository,
                Utc::now(),
            )
            .await
    }

    #[transactional(
        invoice,
        organization,
        project,
        quote,
        customer,
        customer_context,
        role,
        member,
        authz,
        emitter
    )]
    pub async fn issue_final_invoice(
        &self,
        command: IssueFinalInvoiceCommand,
    ) -> Result<InvoiceOutcome<Invoice>, CoreError> {
        let mut organization_repository = organization_repository;
        let mut project_repository = project_repository;
        let mut quote_repository = quote_repository;
        let mut customer_repository = customer_repository;
        let mut customer_context_repository = customer_context_repository;
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
                &mut organization_repository,
                &mut project_repository,
                &mut quote_repository,
                &mut customer_repository,
                &mut customer_context_repository,
                Utc::now(),
            )
            .await
    }

    #[transactional(invoice, organization, role, member, authz, emitter)]
    pub async fn issue_credit_note(
        &self,
        command: IssueCreditNoteCommand,
    ) -> Result<InvoiceOutcome<Invoice>, CoreError> {
        let mut organization_repository = organization_repository;
        let mut service = InvoiceService::new(
            invoice_repository,
            emitter,
            member_repository,
            role_repository,
            authz,
        );
        service
            .issue_credit_note(command, &mut organization_repository, Utc::now())
            .await
    }

    #[transactional(invoice, organization, quote, customer, role, member, authz, emitter)]
    pub async fn create_invoice_from_quote(
        &self,
        command: CreateInvoiceFromQuoteCommand,
    ) -> Result<InvoiceOutcome<Invoice>, CoreError> {
        let mut organization_repository = organization_repository;
        let mut quote_repository = quote_repository;
        let mut customer_repository = customer_repository;
        let mut service = InvoiceService::new(
            invoice_repository,
            emitter,
            member_repository,
            role_repository,
            authz,
        );
        service
            .create_invoice_from_quote(
                command,
                &mut organization_repository,
                &mut quote_repository,
                &mut customer_repository,
                Utc::now(),
            )
            .await
    }

    #[transactional(invoice, quote, role, member, authz, emitter)]
    pub async fn quote_invoicing(
        &self,
        quote_id: QuoteId,
    ) -> Result<QuoteInvoicingView, CoreError> {
        let mut quote_repository = quote_repository;
        let mut service = InvoiceService::new(
            invoice_repository,
            emitter,
            member_repository,
            role_repository,
            authz,
        );
        service
            .quote_invoicing(quote_id, &mut quote_repository, Utc::now().date_naive())
            .await
    }

    #[transactional(invoice, role, member, authz, emitter)]
    pub async fn send_invoice(
        &self,
        command: SendInvoiceCommand,
    ) -> Result<InvoiceOutcome<Invoice>, CoreError> {
        let mut service = InvoiceService::new(
            invoice_repository,
            emitter,
            member_repository,
            role_repository,
            authz,
        );
        service.send_invoice(command, Utc::now()).await
    }

    #[transactional(invoice, role, member, authz, emitter)]
    pub async fn remind_invoice(
        &self,
        command: RemindInvoiceCommand,
    ) -> Result<InvoiceOutcome<Invoice>, CoreError> {
        let mut service = InvoiceService::new(
            invoice_repository,
            emitter,
            member_repository,
            role_repository,
            authz,
        );
        service.remind_invoice(command, Utc::now()).await
    }

    #[transactional(invoice, role, member, authz, emitter)]
    pub async fn note_invoice(
        &self,
        command: NoteInvoiceCommand,
    ) -> Result<InvoiceOutcome<Invoice>, CoreError> {
        let mut service = InvoiceService::new(
            invoice_repository,
            emitter,
            member_repository,
            role_repository,
            authz,
        );
        service.note_invoice(command, Utc::now()).await
    }

    #[transactional(invoice, role, member, authz, emitter)]
    pub async fn record_invoice_payment(
        &self,
        command: RecordInvoicePaymentCommand,
    ) -> Result<InvoiceOutcome<InvoicePayment>, CoreError> {
        let mut service = InvoiceService::new(
            invoice_repository,
            emitter,
            member_repository,
            role_repository,
            authz,
        );
        service.record_payment(command, Utc::now()).await
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
