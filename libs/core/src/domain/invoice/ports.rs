use chrono::{DateTime, Utc};
use common::CoreError;

use crate::{DraftInvoice, Invoice, InvoiceId, InvoiceStatus, OrganizationId, ProjectId};

#[cfg_attr(any(test, feature = "mock"), mockall::automock)]
pub trait InvoiceRepository: Send {
    /// Atomically bumps the organization's counter for `year` and returns
    /// the number it lands on, formatted `{prefix}-{year}-{counter:04}`.
    /// Same locking device as `QuoteRepository::allocate_number` (see
    /// `invoice_number_counters`), deliberately: invoice numbering is
    /// stricter than quote numbering, a gap is a real problem, so it reuses
    /// a pattern already proven safe under contention rather than a new one.
    /// Not exercised by this issue's own service — #317 is the first caller
    /// — but the table and this method exist from #316 so #317 does not
    /// need a migration of its own. `allow(dead_code)` until then: nothing
    /// outside the ignored integration test below calls it yet.
    #[allow(dead_code)]
    fn allocate_number(
        &mut self,
        organization_id: OrganizationId,
        prefix: &str,
        year: i32,
    ) -> impl Future<Output = Result<String, CoreError>> + Send;

    fn insert_draft(
        &mut self,
        draft: &DraftInvoice,
    ) -> impl Future<Output = Result<Invoice, CoreError>> + Send;

    fn find_by_id(
        &mut self,
        id: InvoiceId,
    ) -> impl Future<Output = Result<Option<Invoice>, CoreError>> + Send;

    fn list_by_organization(
        &mut self,
        organization_id: OrganizationId,
        limit: u64,
        offset: u64,
    ) -> impl Future<Output = Result<(Vec<Invoice>, u64), CoreError>> + Send;

    /// Every non-deleted invoice against one project, issued or not — the
    /// read #317 needs to deduct what has already been invoiced, and #322's
    /// project billing summary needs the same list.
    fn list_by_project(
        &mut self,
        project_id: ProjectId,
    ) -> impl Future<Output = Result<Vec<Invoice>, CoreError>> + Send;

    /// Only ever called with a [`DraftInvoice`]: the type that carries the
    /// guarantee this row is still a draft.
    fn update_draft(
        &mut self,
        draft: &DraftInvoice,
    ) -> impl Future<Output = Result<Invoice, CoreError>> + Send;

    /// A bare status transition, carrying nothing about content — used by
    /// `cancel_invoice` here. Issuing (#317) and recording a payment (#320)
    /// need more than a status (a number, a frozen identity, recomputed
    /// totals) and get their own port methods rather than overloading this
    /// one.
    fn update_status(
        &mut self,
        id: InvoiceId,
        status: InvoiceStatus,
        updated_at: DateTime<Utc>,
    ) -> impl Future<Output = Result<Invoice, CoreError>> + Send;

    /// Refused by the service unless the invoice is still a draft: an
    /// issued invoice is a legal document, corrected with a credit note
    /// (#318), never deleted.
    fn soft_delete(
        &mut self,
        id: InvoiceId,
        deleted_at: DateTime<Utc>,
    ) -> impl Future<Output = Result<(), CoreError>> + Send;
}
