use common::CoreError;
use mestier_macros::transactional;

use crate::{
    OrganizationId, SupplierInvoice, SupplierInvoiceId,
    application::MestierUseCase,
    domain::supplier_invoice::{
        commands::{
            ConfirmSupplierInvoiceCommand, CreateSupplierInvoiceCommand,
            RejectSupplierInvoiceCommand,
        },
        service::SupplierInvoiceService,
    },
};

impl MestierUseCase {
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
}
