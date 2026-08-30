use authz::Subject;
use chrono::{DateTime, NaiveDate, Utc};
use common::UserId;
use rust_decimal::Decimal;

use crate::{
    CustomerContextId, CustomerId, InvoiceId, InvoiceKind, InvoicePaymentId, OperationNature,
    OrganizationAddress, OrganizationId, ProjectId,
};

/// Issues any draft invoice, whatever `InvoiceKind` built it — a manually
/// created `Standard` draft from #316's `create_invoice`, or a
/// `Deposit`/`Final` draft from this issue's own `issue_deposit`/
/// `issue_final_invoice`. See `InvoiceService::issue_invoice`.
///
/// No `organization_id` of its own (#395): the service loads the invoice
/// first, the same way `CancelInvoiceCommand`'s own act does, and enriches
/// `actor` against the loaded row's own organization.
#[derive(Debug, Clone)]
pub struct IssueInvoiceCommand {
    pub id: InvoiceId,
    pub actor: Subject,
    /// Refuses issuing when it would push the project's issued total past
    /// the quote, unless the caller explicitly says this is fine: a
    /// deliberate over-bill happens, the artisan should not have to lie to
    /// the system to do it, but the system should not do it silently
    /// either.
    pub allow_exceeding_total: bool,
}

/// Builds a deposit invoice for a percentage of the project's quoted total
/// and issues it in one step. Needs a quote to compute a percentage
/// against — a general limitation of this specific act, not of invoicing
/// as a whole (`create_invoice`/`issue_invoice` still work with no
/// project, or a project with no quote).
///
/// No `organization_id` of its own (#395): the service loads the project
/// first (it needs it anyway, for the quote) and enriches `actor` against
/// the loaded project's own organization.
#[derive(Debug, Clone)]
pub struct IssueDepositCommand {
    pub project_id: ProjectId,
    pub actor: Subject,
    /// Basis points of the quote's net total, e.g. 3000 = 30%.
    pub percentage_bp: i32,
    pub due_at: Option<DateTime<Utc>>,
    pub notes: Option<String>,
    pub allow_exceeding_total: bool,
}

/// Builds an invoice for exactly what remains to bill on the project's
/// quote — `remaining_to_bill_cents` after every already-issued,
/// non-cancelled invoice — and issues it in one step. Same quote
/// requirement, and the same reason, as `IssueDepositCommand`.
#[derive(Debug, Clone)]
pub struct IssueFinalInvoiceCommand {
    pub project_id: ProjectId,
    pub actor: Subject,
    pub due_at: Option<DateTime<Utc>>,
    pub notes: Option<String>,
    pub allow_exceeding_total: bool,
}

#[derive(Debug, Clone)]
pub struct InvoiceLineCommand {
    pub label: String,
    pub quantity: Decimal,
    pub unit_price_cents: i32,
    pub vat_rate_basis_points: Option<i32>,
}

#[derive(Debug, Clone)]
pub struct CreateInvoiceCommand {
    pub actor: Subject,
    pub organization_id: OrganizationId,
    pub kind: InvoiceKind,
    pub project_id: Option<ProjectId>,
    pub customer_id: CustomerId,
    pub customer_context_id: CustomerContextId,
    pub due_at: Option<DateTime<Utc>>,
    pub notes: Option<String>,
    pub operation_nature: Option<OperationNature>,
    pub delivery_address: Option<OrganizationAddress>,
    pub lines: Vec<InvoiceLineCommand>,
}

#[derive(Debug, Clone)]
pub struct UpdateInvoiceCommand {
    pub id: InvoiceId,
    pub actor: Subject,
    pub project_id: Option<ProjectId>,
    pub customer_id: CustomerId,
    pub customer_context_id: CustomerContextId,
    pub due_at: Option<DateTime<Utc>>,
    pub notes: Option<String>,
    pub operation_nature: Option<OperationNature>,
    pub delivery_address: Option<OrganizationAddress>,
    pub lines: Vec<InvoiceLineCommand>,
}

/// Issues a credit note against a source invoice — the only way to correct
/// an issued invoice (#318), since `Invoice` itself has no mutating methods.
/// The credit note's `customer_id`/`customer_context_id`/`project_id` are
/// not on this command: they are copied from the source invoice, never
/// supplied by the caller, because a credit note corrects one specific
/// document and its counterparty cannot differ from it.
#[derive(Debug, Clone)]
pub struct IssueCreditNoteCommand {
    pub source_invoice_id: InvoiceId,
    pub actor: Subject,
    pub lines: Vec<InvoiceLineCommand>,
    pub notes: Option<String>,
    /// Refuses when the sum of this credit note and every other
    /// non-draft, non-cancelled credit note already issued against the
    /// same source would exceed the source invoice's net_cents, unless
    /// the caller explicitly says this is fine.
    pub allow_exceeding_invoice_total: bool,
}

/// Cancels an invoice, draft or issued. Never used to reach `Issued`,
/// `Paid` or `PartiallyPaid`: those are set by issuing (#317) and by
/// recording payments (#320), never by hand.
///
/// No `organization_id` of its own (#395): the service loads the invoice
/// first (it needs its current status anyway) and enriches `actor` against
/// the loaded row's own organization, same as `CustomerService::
/// soft_delete_customer` does for a bare id.
#[derive(Debug, Clone)]
pub struct CancelInvoiceCommand {
    pub id: InvoiceId,
    pub actor: Subject,
}

/// Records a payment against an issued invoice (#320).
#[derive(Debug, Clone)]
pub struct RecordInvoicePaymentCommand {
    pub invoice_id: InvoiceId,
    pub actor: Subject,
    pub amount_cents: i32,
    pub paid_on: NaiveDate,
    pub method: String,
    pub reference: Option<String>,
    pub note: Option<String>,
    pub recorded_by: UserId,
    /// Refuses when this payment would push the recorded total past the
    /// invoice's gross amount net of credit notes, unless explicitly
    /// allowed — mirrors `allow_exceeding_total` on the issuing commands.
    pub allow_exceeding_total: bool,
}

/// Soft-deletes a recorded payment, keeping who and when as its audit
/// trail (#320) — the row is never hard-deleted.
///
/// No `organization_id` of its own (#395): the service loads the payment
/// first (`InvoicePayment` carries its own `organization_id`, needed
/// anyway to resolve it) and enriches `actor` against it.
#[derive(Debug, Clone)]
pub struct DeleteInvoicePaymentCommand {
    pub id: InvoicePaymentId,
    pub actor: Subject,
    pub deleted_by: UserId,
}
