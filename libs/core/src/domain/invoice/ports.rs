use chrono::{DateTime, Utc};
use common::{CoreError, UserId};

use crate::{
    CustomerOutstandingBalance, DraftInvoice, Invoice, InvoiceId, InvoicePayment, InvoicePaymentId,
    InvoiceStatus, LegalIdentity, OrganizationId, ProjectId,
};

#[cfg_attr(any(test, feature = "mock"), mockall::automock)]
pub trait InvoiceRepository: Send {
    /// Atomically bumps the organization's counter for `year` and returns
    /// the number it lands on, formatted `{prefix}-{year}-{counter:04}`.
    /// Same locking device as `QuoteRepository::allocate_number` (see
    /// `invoice_number_counters`), deliberately: invoice numbering is
    /// stricter than quote numbering, a gap is a real problem, so it reuses
    /// a pattern already proven safe under contention rather than a new one.
    /// The table and this method exist from #316; #317's `issue_invoice` is
    /// the first production caller.
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

    /// Every non-deleted credit note referencing one source invoice —
    /// what `InvoiceService::issue_credit_note`'s own limit check sums, and
    /// what `net_of_credit_notes_cents` reads to tell "fully credited" (#318).
    fn list_by_source_invoice(
        &mut self,
        source_invoice_id: InvoiceId,
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

    /// The transition out of `Draft`: sets the allocated number, the frozen
    /// issuer identity, `issued_at`, and the `Issued` status, all at once.
    /// Deliberately not folded into `update_status` — issuing carries
    /// content, not a bare status — and not reachable through
    /// `update_draft`, which by design cannot leave `Draft`. Only ever
    /// called against a row still `Draft`; the implementation's `WHERE`
    /// clause enforces that independently of the service-level check.
    fn issue(
        &mut self,
        id: InvoiceId,
        number: String,
        issued_at: DateTime<Utc>,
        issuer_identity: &LegalIdentity,
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

    /// A payment recorded against an invoice (#320) — a sub-resource of
    /// this same aggregate, not a new repository. Not in the issue's own
    /// file list, but unavoidable, same as #317 and #318 both had to
    /// extend this trait too.
    fn insert_payment(
        &mut self,
        payment: &InvoicePayment,
    ) -> impl Future<Output = Result<InvoicePayment, CoreError>> + Send;

    /// `None` for a payment that never existed, or was already
    /// soft-deleted: same "not found or deleted reads as absent" rule
    /// every other `find_by_id` in this codebase follows.
    fn find_payment_by_id(
        &mut self,
        id: InvoicePaymentId,
    ) -> impl Future<Output = Result<Option<InvoicePayment>, CoreError>> + Send;

    /// Every non-deleted payment against one invoice, ordered by
    /// `paid_on` then `created_at` — what `derive_invoice_status` and
    /// `InvoiceService::record_payment`'s own limit check both read.
    fn list_payments(
        &mut self,
        invoice_id: InvoiceId,
    ) -> impl Future<Output = Result<Vec<InvoicePayment>, CoreError>> + Send;

    /// The audit trail #320 asks for: the row survives with who deleted it
    /// and when, it is never hard-deleted.
    fn soft_delete_payment(
        &mut self,
        id: InvoicePaymentId,
        deleted_at: DateTime<Utc>,
        deleted_by: UserId,
    ) -> impl Future<Output = Result<(), CoreError>> + Send;

    /// One row per customer with an outstanding balance, computed as a SQL
    /// aggregate — the dunning list #320 asks for. See
    /// `CustomerOutstandingBalance`.
    fn list_outstanding_by_customer(
        &mut self,
        organization_id: OrganizationId,
    ) -> impl Future<Output = Result<Vec<CustomerOutstandingBalance>, CoreError>> + Send;

    /// Every issued invoice past its due date with a balance still owed as
    /// of `as_of`, oldest first.
    fn list_overdue(
        &mut self,
        organization_id: OrganizationId,
        as_of: DateTime<Utc>,
    ) -> impl Future<Output = Result<Vec<Invoice>, CoreError>> + Send;
}
