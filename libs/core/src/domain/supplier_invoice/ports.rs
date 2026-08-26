use common::CoreError;

use crate::{OrganizationId, SupplierInvoice, SupplierInvoiceId, SupplierInvoiceReview};

#[cfg_attr(any(test, feature = "mock"), mockall::automock)]
pub trait SupplierInvoiceRepository: Send {
    fn insert(
        &mut self,
        invoice: &SupplierInvoice,
    ) -> impl Future<Output = Result<SupplierInvoice, CoreError>> + Send;

    fn find_by_id(
        &mut self,
        id: SupplierInvoiceId,
    ) -> impl Future<Output = Result<Option<SupplierInvoice>, CoreError>> + Send;

    fn list_by_organization(
        &mut self,
        organization_id: OrganizationId,
        limit: u64,
        offset: u64,
    ) -> impl Future<Output = Result<(Vec<SupplierInvoice>, u64), CoreError>> + Send;

    /// Only ever called with a [`SupplierInvoiceReview`]: persists `status`
    /// and `notes` alone, never the document's own fields. The type is what
    /// keeps a call site holding the wrong thing from compiling; the
    /// implementation's `UPDATE` naming exactly those two columns is what
    /// keeps it true even if a future caller reaches for the struct
    /// directly.
    fn update_review(
        &mut self,
        review: &SupplierInvoiceReview,
    ) -> impl Future<Output = Result<SupplierInvoice, CoreError>> + Send;
}
