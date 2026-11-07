use chrono::Utc;
use common::CoreError;
use mestier_macros::transactional;

use crate::{
    InvoiceOutcome, InvoicePayment, Ledger, LedgerMonth, LedgerOutcome, OrganizationId, Receipt,
    ReceiptId,
    application::MestierUseCase,
    domain::{
        invoice::{commands::RecordInvoicePaymentCommand, service::InvoiceService},
        ledger::{
            LedgerService, ReceiptService,
            commands::{AllocateReceiptCommand, RecordReceiptCommand},
            ports::ReceiptRepository,
        },
    },
};

impl MestierUseCase {
    #[transactional(ledger, expense)]
    pub async fn read_ledger(
        &self,
        organization_id: OrganizationId,
        month: Option<LedgerMonth>,
    ) -> Result<Ledger, CoreError> {
        let mut service = LedgerService::new(ledger_repository, expense_repository);
        service
            .read(organization_id, month, Utc::now().date_naive())
            .await
    }

    #[transactional(receipt, customer, role, member, authz)]
    pub async fn record_receipt(
        &self,
        command: RecordReceiptCommand,
    ) -> Result<LedgerOutcome<Receipt>, CoreError> {
        let mut service = ReceiptService::new(
            receipt_repository,
            customer_repository,
            member_repository,
            role_repository,
            authz,
        );
        service.record_receipt(command, Utc::now()).await
    }

    #[transactional(receipt, customer, role, member, authz)]
    pub async fn get_receipt(&self, id: ReceiptId) -> Result<Receipt, CoreError> {
        let mut service = ReceiptService::new(
            receipt_repository,
            customer_repository,
            member_repository,
            role_repository,
            authz,
        );
        service.get_receipt(id).await
    }

    #[transactional(receipt, invoice, role, member, authz, emitter)]
    pub async fn allocate_receipt(
        &self,
        command: AllocateReceiptCommand,
    ) -> Result<InvoiceOutcome<InvoicePayment>, CoreError> {
        let mut receipts = receipt_repository;
        let receipt = receipts
            .find_by_id(command.id)
            .await?
            .ok_or(CoreError::NotFound)?;

        let mut service = InvoiceService::new(
            invoice_repository,
            emitter,
            member_repository,
            role_repository,
            authz,
        );
        let outcome = service
            .record_payment(
                RecordInvoicePaymentCommand {
                    invoice_id: command.invoice_id,
                    actor: command.actor,
                    amount_cents: receipt.amount_cents,
                    paid_on: receipt.paid_on,
                    method: receipt.method,
                    reference: receipt.reference,
                    note: receipt.note,
                    recorded_by: command.recorded_by,
                },
                Utc::now(),
            )
            .await?;

        match outcome {
            Ok(payment) if payment.organization_id == receipt.organization_id => {
                receipts.delete(receipt.id).await?;
                Ok(Ok(payment))
            }
            Ok(_) => Err(CoreError::NotFound),
            Err(refusal) => Ok(Err(refusal)),
        }
    }
}
