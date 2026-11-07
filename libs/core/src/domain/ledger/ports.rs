use common::CoreError;

use super::{DocumentVatRow, ReceiptRow};
use crate::{OrganizationId, Receipt, ReceiptId};

#[cfg_attr(any(test, feature = "mock"), mockall::automock)]
pub trait LedgerRepository: Send {
    fn receipts(
        &mut self,
        organization_id: OrganizationId,
    ) -> impl Future<Output = Result<Vec<ReceiptRow>, CoreError>> + Send;

    fn document_vat(
        &mut self,
        organization_id: OrganizationId,
    ) -> impl Future<Output = Result<Vec<DocumentVatRow>, CoreError>> + Send;
}

#[cfg_attr(any(test, feature = "mock"), mockall::automock)]
pub trait ReceiptRepository: Send {
    fn insert(
        &mut self,
        receipt: &Receipt,
    ) -> impl Future<Output = Result<Receipt, CoreError>> + Send;

    fn find_by_id(
        &mut self,
        id: ReceiptId,
    ) -> impl Future<Output = Result<Option<Receipt>, CoreError>> + Send;

    fn delete(&mut self, id: ReceiptId) -> impl Future<Output = Result<(), CoreError>> + Send;
}
