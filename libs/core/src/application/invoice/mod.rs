use common::CoreError;
use mestier_macros::transactional;

use crate::{
    Invoice, InvoiceId, OrganizationId, ProjectId,
    application::MestierUseCase,
    domain::invoice::{
        commands::{CancelInvoiceCommand, CreateInvoiceCommand, UpdateInvoiceCommand},
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
}
